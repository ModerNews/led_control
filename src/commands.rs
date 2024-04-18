pub mod commmands {
    use async_std::io::WriteExt;
    use async_std::net::{TcpStream, UdpSocket};
    use futures::AsyncReadExt;
    use std::error::Error;

    #[derive(Clone, Copy)]
    pub enum Commands {
        On,
        Off,
        GetStatus,
        SetColor(u8, u8, u8),
    }

    pub async fn wake_signal() -> Result<(), Box<dyn Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await.expect("Failed to bind");
        socket.set_broadcast(true).expect("Failed to set broadcast");
        let addr = match socket.local_addr() {
            Ok(addr) => addr,
            Err(e) => return Err(e.into()),
        };
        println!("Local address: {:?}", addr);
        println!("Broadcast: {:?}", socket.broadcast());

        let buf = b"HF-A11ASSISTHREAD";

        socket
            .connect("255.255.255.255:48899")
            .await
            .expect("Failed to connect");
        socket.send(buf).await.expect("Failed to send data");
        Ok(())
    }

    pub struct Strip {
        socket: Option<TcpStream>,
        color: (u8, u8, u8),
        powered: bool,
        address: String,
        is_rgbw: bool,
    }

    impl Strip {
        // Constructor based on IP address of the strip
        pub async fn new(address: String, is_rgbw: Option<bool>) -> Self {
            let mut tmp_strip = Strip {
                socket: None,
                color: (0, 0, 0),
                powered: false,
                is_rgbw: is_rgbw.unwrap_or(false),
                address,
            };
            let _ = tmp_strip.connect().await;
            let _ = tmp_strip.execute(&Commands::GetStatus).await;
            tmp_strip
        }

        async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
            let socket = TcpStream::connect(&self.address).await?;
            self.socket = Some(socket);
            Ok(())
        }

        fn parse_state(&mut self, data: Vec<u8>) {
            self.powered = data[2] == 0x23;
            self.color = (data[6], data[7], data[8]);
        }

        pub async fn execute(&mut self, &command: &Commands) -> Result<String, Box<dyn Error>> {
            // TODO: Wait for response with 0-length payload
            if let Some(socket) = &mut self.socket {
                match command {
                    Commands::On => {
                        let buf = vec![0x71, 0x23];
                        socket.write_all(&buf).await.expect("Failed to send data");
                        let mut buf = vec![0u8; 0];
                        socket
                            .read_exact(&mut buf)
                            .await
                            .expect("Failed to read data");
                        Ok(String::from_utf8(buf).expect("Failed to convert to string"))
                    }
                    Commands::Off => {
                        let buf = vec![0x71, 0x24];
                        socket.write_all(&buf).await.expect("Failed to send data");

                        let mut buf = vec![0u8; 0];
                        socket
                            .read_exact(&mut buf)
                            .await
                            .expect("Failed to read data");
                        Ok(String::from_utf8(buf).expect("Failed to convert to string"))
                    }
                    Commands::GetStatus => {
                        let buf = vec![0x81, 0x8a, 0x8b, 0x96];
                        socket.write_all(&buf).await.expect("Failed to send data");
                        let mut buf = vec![0u8; 14];
                        socket
                            .read_exact(&mut buf)
                            .await
                            .expect("Failed to read data");
                        let response = buf.clone();
                        self.parse_state(buf.to_vec().clone());
                        Ok(response
                            .iter()
                            .map(|&byte| format!("{:02x}", byte))
                            .collect::<Vec<_>>()
                            .join(":"))
                    }
                    Commands::SetColor(r, g, b) => {
                        let buf = vec![0x31, r, g, b, 0x00, 0x0f, 0xff, 0x00];
                        socket.write_all(&buf).await.expect("Failed to send data");

                        let mut buf = vec![0u8; 0];
                        socket
                            .read_exact(&mut buf)
                            .await
                            .expect("Failed to read data");
                        Ok(buf
                            .iter()
                            .map(|&byte| format!("{:02x}", byte))
                            .collect::<Vec<_>>()
                            .join(":"))
                    }
                }
            } else {
                Err("No socket available...".into())
            }
        }
    }
}
