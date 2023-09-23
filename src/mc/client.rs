use crate::error::DshError;
use crate::tf::token::Token;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, Outgoing, PubAck, QoS, Transport};
use rustls::ClientConfig;
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct Client {
    client_id: String,
    broker_url: String,
    port: u16,
    token: String,
    topic: String,
    message: Option<String>,
    websocket: bool,
    verbose: bool,
    concise: bool,
}

impl Client {
    pub async fn new(
        token: Token,
        port: u16,
        topic: String,
        websocket: bool,
        verbose: bool,
        concise: bool,
        message: Option<String>,
    ) -> Result<Client, DshError> {
        // format the url for the broker depending on the protocol
        let broker_url = if websocket {
            format!("wss://{}/mqtt", &token.token_attributes.endpoint)
        } else {
            token.token_attributes.endpoint.clone()
        };

        // check if port is present in the token
        if websocket {
            if !token.token_attributes.ports.mqttwss.contains(&port) {
                return Err(DshError::PortNotPresentInToken(port));
            }
        } else {
            if !token.token_attributes.ports.mqtts.contains(&port) {
                return Err(DshError::PortNotPresentInToken(port));
            }
        }

        Ok(Self {
            client_id: token.token_attributes.client_id.clone(),
            broker_url,
            port,
            token: token.raw_token,
            topic,
            message,
            websocket,
            verbose,
            concise,
        })
    }

    pub async fn connect(&self) -> Result<(), DshError> {
        let mut mqttoptions = MqttOptions::new(&self.client_id, &self.broker_url, self.port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        // load (OS) tls certs
        let mut root_cert_store = rustls::RootCertStore::empty();
        for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs")
        {
            root_cert_store.add(&rustls::Certificate(cert.0)).unwrap();
        }

        // secure client config
        let client_config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        // if websockets are used
        if self.websocket {
            info!("Websockets will be used");
            mqttoptions.set_transport(Transport::Wss(client_config.into()));
        } else {
            info!("Tcp will be used (no websockets)");
            mqttoptions.set_transport(Transport::tls_with_config(client_config.into()));
        }

        // set tls options and credentials
        mqttoptions.set_credentials(&self.client_id, &self.token);
        debug!("{:?}", &mqttoptions);

        info!("Config: {:?}", self);
        // check if there is only a message to be pushed
        match &self.message {
            Some(message) => {
                Self::publish_message_to_topic(self, mqttoptions, message.to_owned()).await?
            }
            None => Self::subscribe_to_topic(self, mqttoptions).await?,
        }

        info!("Connection closed");

        Ok(())
    }

    async fn publish_message_to_topic(
        &self,
        mqttoptions: MqttOptions,
        message: String,
    ) -> Result<(), DshError> {
        info!("New client, getting an async connection");
        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        Self::publish_message(&client, self.topic.clone(), message).await?;

        // listen to messages to see if we received an acknoledgement that the message was published
        loop {
            match eventloop.poll().await {
                // Publish acknowledgement
                Ok(Event::Incoming(Incoming::PubAck(PubAck { pkid: 1 }))) => {
                    println!("Message published");
                    break;
                }
                // other Ok events
                Ok(e) => {
                    println!("Event: {:?}", e);
                }
                // Errors
                Err(e) => {
                    error!("Error while polling received messages: {:?}", e);
                    break;
                }
            }
        }

        info!("Stop publishing");

        Ok(())
    }

    async fn subscribe_to_topic(&self, mqttoptions: MqttOptions) -> Result<(), DshError> {
        info!("New client, getting an async connection");
        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        info!("Subscribing to topic \"{}\":... ", &self.topic);
        client.subscribe(&self.topic, QoS::AtLeastOnce).await?;

        // so the verbose input can be moved to an other thread
        let verbose_input = self.verbose;
        let concise_input = self.concise;

        let rt = Runtime::new()?;
        thread::spawn(move || {
            rt.block_on(async {
                loop {
                    match eventloop.poll().await {
                        Ok(notification) => {
                            // show payload of received messages
                            if let Event::Incoming(Incoming::Publish(publish)) = &notification {
                                if !concise_input {
                                    println!("Event: {:?}", notification);
                                    println!(
                                        "Decoded message: {}",
                                        String::from_utf8_lossy(&publish.payload)
                                    );
                                } else {
                                    println!(
                                        "{} > {}",
                                        &publish.topic,
                                        String::from_utf8_lossy(&publish.payload)
                                    );
                                }
                            } else if notification == Event::Outgoing(Outgoing::PingReq)
                                || notification == Event::Incoming(Incoming::PingResp)
                            {
                                if verbose_input {
                                    println!("Event: {:?}", notification);
                                }
                            } else {
                                if !concise_input {
                                    println!("Event: {:?}", notification);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error while polling received messages: {:?}", e);
                            break;
                        }
                    }
                }
            })
        });

        // Read input from the CLI in the main thread
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input = input.trim().to_string();

            if input == "exit" {
                info!("Exiting...");
                break;
            } else {
                Self::publish_message(&client, self.topic.clone(), input).await?;
            }
        }

        Ok(())
    }

    async fn publish_message(
        client: &AsyncClient,
        topic: String,
        message: String,
    ) -> Result<(), DshError> {
        // remove '#' and '+' from topic if this exists
        let topic = topic.replace("#", "").replace("+", "");

        info!("Publishing message...");
        client
            .publish(topic, QoS::AtLeastOnce, true, message)
            .await?;

        Ok(())
    }
}
