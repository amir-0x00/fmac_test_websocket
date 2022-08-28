use std::{sync::Arc, time::Duration};

use actix::clock::sleep;
use actix_codec::Framed;
use actix_web::web::Bytes;
use awc::ws::{self, Codec};
use futures::StreamExt;
use futures_util::SinkExt;
use once_cell::sync::Lazy;
use tokio::sync::{
    mpsc::{self, Sender, UnboundedReceiver, UnboundedSender},
    Mutex,
};
use tokio_stream::wrappers::UnboundedReceiverStream;

// pub const DEFAULT_IP: &str = "94.237.92.96";
pub const DEFAULT_IP: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 6379;

// type ArcChannel = (UnboundedSender<String>, UnboundedReceiver<String>);
// static SERVER_CHANNEL: Lazy<ArcChannel> = Lazy::new(|| {
//     let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
//     // cmd_rx.recv();
//     // let mut cmd_rx = UnboundedReceiverStream::new(cmd_rx);
//     (cmd_tx, cmd_rx)
// });

static SERVER_SENDER: Lazy<Mutex<Option<UnboundedSender<SenderMsg>>>> =
    Lazy::new(|| Mutex::new(None));
// static mut SERVER_SENDER: Mutex<Option<UnboundedSender<String>>> = Mutex::new(None);

#[derive(Debug, Clone)]
pub enum SenderMsg {
    Disconnected,
    Text(String),
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let mut cmd_rx = UnboundedReceiverStream::new(cmd_rx);
    *SERVER_SENDER.lock().await = Some(cmd_tx);

    let mut try_count = 0;
    loop {
        let millis = (200 * try_count).min(2500);
        println!("retry : {:?}", millis);
        let duration = Duration::from_millis(millis);
        tokio::time::sleep(duration).await;
        let is_connected = start(DEFAULT_IP, DEFAULT_PORT, &mut cmd_rx).await;

        if is_connected {
            try_count = 0;
        } else {
            try_count += 1;
        }
    }

    // TcpServer::start(DEFAULT_IP, DEFAULT_PORT).await?;

    // tokio::spawn(async {
    //     loop {
    //         if let Some(cmd) = cmd_rx.next().await {
    //             if cmd.is_empty() {
    //                 continue;
    //             }

    //             ws.send(ws::Message::Text(cmd.into())).await.unwrap();
    //         }
    //     }
    // });

    // println!("#####");
    // input_thread.join().unwrap();
    // Ok(())
}

async fn start(ip: &str, port: u16, cmd_rx: &mut UnboundedReceiverStream<SenderMsg>) -> bool {
    let connection = awc::Client::new()
        .ws(format!("ws://{ip}:{port}/ws"))
        .connect()
        .await;

    if let Ok((res, ws)) = connection {
        println!("response: {res:?}");
        println!("connected; server will echo messages sent");

        // let mut cmd_rx = UnboundedReceiverStream::new(SERVER_CHANNEL.1);
        let (sink, mut stream) = ws.split();

        let ws_sender = Arc::new(Mutex::new(sink));

        tokio::join!(
            async {
                loop {
                    if let Some(msg) = stream.next().await {
                        match msg {
                            Ok(ws::Frame::Text(txt)) => {
                                // log echoed messages from server
                                println!("Server: {txt:?}")
                            }

                            Ok(ws::Frame::Ping(_)) => {
                                // respond to ping probes
                                let mut ws = ws_sender.lock().await;
                                ws.send(ws::Message::Pong(Bytes::new())).await.unwrap();
                            }

                            _ => {
                                println!("ERRR");
                            }
                        }
                    } else {
                        let mut ws = SERVER_SENDER.lock().await;
                        if let Some(ws) = ws.as_mut() {
                            ws.send(SenderMsg::Disconnected).unwrap();
                        }
                        // ws.send(SenderMsg::Disconnected).unwrap();
                        println!("ERRRxXXXXX");
                        break;
                    }
                }
            },
            async {
                loop {
                    if let Some(msg) = cmd_rx.next().await {
                        match msg {
                            SenderMsg::Disconnected => break,
                            SenderMsg::Text(msg_str) => {
                                let mut ws = ws_sender.lock().await;
                                ws.send(ws::Message::Text(msg_str.into())).await.unwrap();
                            }
                        }
                    }
                }
            },
            // async {
            //     loop {
            //         sleep(Duration::from_millis(1000)).await;
            //         println!("1");
            //         let mut ws = SERVER_SENDER.lock().await;
            //         if let Some(ws) = ws.as_mut() {
            //             ws.send(SenderMsg::Text("#".to_owned())).unwrap();
            //         }
            //         // let mut ws = ws_sender.lock().await;
            //         // ws.send(ws::Message::Text("cmd".into())).await.unwrap();
            //     }
            // }
        );
        return true;
    }
    false
}
// #[derive(Debug)]
// pub struct TcpServer {
//     is_running: bool,
//     stream: Option<TcpStream>,
// }

// impl TcpServer {
//     fn empty() -> Self {
//         Self {
//             is_running: false,
//             stream: None,
//         }
//     }
//     // fn stop(&mut self) {
//     //     self.is_running = false;
//     //     self.stream = None;
//     // }

//     fn is_connected(&self) -> bool {
//         self.stream.is_some()
//     }
//     // fn new(stream: TcpStream) -> Self {
//     //     Self { stream }
//     // }
//     pub async fn start(ip: &str, port: u16) -> std::io::Result<()> {
//         // loop {
//         let stream = Self::connect(ip, port).await;
//         //     {
//         //         let mut server_instance = TCP_SERVER_STREAM.write().await;
//         //         server_instance.stream = Some(stream);
//         //         server_instance.is_running = true;
//         //     }
//         //     Self::start_listener().await?;
//         // }

//         Ok(())
//     }

//     async fn connect(ip: &str, port: u16) -> Framed<Box<dyn ConnectionIo>, Codec> {
//         let mut try_count = 0;
//         loop {
//             let millis = (200 * try_count).min(2500);
//             println!("retry : {:?}", millis);
//             let duration = Duration::from_millis(millis);
//             tokio::time::sleep(duration).await;

//             let response = awc::Client::new()
//                 .ws("ws://{DEFAULT_IP}:{DEFAULT_PORT}/ws")
//                 .connect()
//                 .await;
//             if let Ok((res, ws)) = response {
//                 return ws;
//             } else {
//                 try_count += 1;
//             }
//         }
//     }

//     // async fn start_listener() -> std::io::Result<()> {
//     //     println!("start_listener");
//     //     let t1 = tokio::spawn(async move {
//     //         loop {
//     //             let server = TCP_SERVER_STREAM.read().await;
//     //             if !server.is_running {
//     //                 break;
//     //             }
//     //             if let Some(ref stream) = server.stream {
//     //                 let ready = stream
//     //                     .ready(Interest::READABLE | Interest::WRITABLE)
//     //                     .await
//     //                     .unwrap();
//     //                 if ready.is_readable() {
//     //                     let mut data = vec![0; 1024];
//     //                     // Try to read data, this may still fail with `WouldBlock`
//     //                     // if the readiness event is a false positive.
//     //                     match stream.try_read(&mut data) {
//     //                         Ok(len) => {
//     //                             if len == 0 {
//     //                                 println!("{}", "closed".red());
//     //                                 break;
//     //                             }
//     //                             // println!("len_buffer : {:?}", len_buffer);
//     //                             // let len = u32::from_be_bytes(len_buffer);
//     //                             println!("len: {:?}", len);
//     //                         }
//     //                         Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
//     //                             continue;
//     //                         }
//     //                         Err(_e) => {
//     //                             // return Err(e.into());
//     //                             break;
//     //                         }
//     //                     }
//     //                 }
//     //             } else {
//     //                 break;
//     //             }
//     //         }
//     //     });

//     //     let t2 = tokio::spawn(async move {
//     //         // let duration = Duration::from_millis(1000);
//     //         // tokio::time::sleep(duration).await;
//     //         let server = TCP_SERVER_STREAM.read().await;

//     //         if let Some(ref stream) = server.stream {
//     //             let _ = stream.try_write(b"amir").unwrap();
//     //             let _ = stream.try_write(b"xmir").unwrap();
//     //         }
//     //     });

//     //     let t3 = tokio::spawn(async move {
//     //         // let duration = Duration::from_millis(1500);
//     //         // tokio::time::sleep(duration).await;
//     //         let mut server = TCP_SERVER_STREAM.write().await;
//     //         if let Some(ref mut stream) = server.stream {
//     //             let _ = stream.try_write(b"walid").unwrap();
//     //         }

//     //         // server.stop();
//     //     });
//     //     // let t2 = tokio::spawn(async move {
//     //     //     println!("tokio::spawn");
//     //     //     let mut buffer = [0; 1024];
//     //     //     loop {
//     //     //         let mut server = TCP_SERVER_STREAM.read().await;
//     //     //         if let Some(stream) = server.stream.as_mut() {
//     //     //             let len = if let Ok(len) = stream.read(&mut buffer).await {
//     //     //                 len
//     //     //             } else {
//     //     //                 0
//     //     //             };
//     //     //             if len > 0 {
//     //     //                 // let message = String::from_utf8_lossy(&buffer[..len]);
//     //     //                 let len_buffer: [u8; 4] = buffer[..4].try_into().unwrap();
//     //     //                 println!("len_buffer : {:?}", len_buffer);
//     //     //                 let len = u32::from_be_bytes(len_buffer);
//     //     //                 println!("len: {:?}", len);
//     //     //                 // let message = u16::from_be_bytes(len_buffer); // println!("message : {:?}", message.as_bytes());
//     //     //                 // let _ = stream.write(message.as_bytes()).await;
//     //     //                 // let _ = stream.flush().await;
//     //     //             } else {
//     //     //                 println!("{}", "closed".red());
//     //     //                 // stream has reached EOF
//     //     //                 break;
//     //     //             }
//     //     //         } else {
//     //     //             break;
//     //     //         }
//     //     //         // println!("server : {:?}", stream.peer_addr());

//     //     //         // tokio::select! {
//     //     //         //   _ = timeout.as_mut() =>{
//     //     //         //     // let message = math_rand_alpha(15);
//     //     //         //     // println!("Sending '{}'", message);
//     //     //         //     // result = d2.send_text(message).await.map_err(Into::into);
//     //     //         //     let _ = stream.write_all(b"2222").await;
//     //     //         //     println!("sent : {:?}", stream.local_addr());
//     //     //         //   }
//     //     //     }
//     //     // });
//     //     t1.await?;
//     //     // tokio::join!(t1, t2);
//     //     // tokio::join!(t1, t2);
//     //     Ok(())
//     // }
// }
