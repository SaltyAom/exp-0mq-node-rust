use scylla::{ IntoTypedRows, SessionBuilder };
use std::error::Error;
use zeromq::{Socket, SocketRecv, SocketSend};
use std::sync::Arc;
use tokio::sync::Mutex;

async fn setup() -> Result<scylla::Session, Box<dyn Error>> {
    let uri = std::env::var("SCYLLA_URI")
        .unwrap_or_else(|_| "127.0.0.1:9042".to_string());

    let session = SessionBuilder::new()
        .known_node(uri)
        .compression(Some(scylla::transport::Compression::Lz4))
        .build()
        .await?;

    session
        .query(
            "CREATE KEYSPACE IF NOT EXISTS ks WITH REPLICATION = \
            {'class' : 'SimpleStrategy', 'replication_factor' : 1}",
            &[],
        )
        .await?;

    session
        .query(
            "CREATE TABLE IF NOT EXISTS ks.salt1 (username ASCII primary key, password ASCII)",
            &[],
        )
        .await?;

    session
        .query(
            "INSERT INTO ks.salt1 (username, password) VALUES (?, ?)", 
            ("saltyaom", "12345678")
        )
        .await?;

    Ok(session)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let scylla = setup()
        .await
        .expect("Failed to setup session");

    let arc_query = Arc::new(scylla.prepare(
        "SELECT username, password FROM ks.salt1 WHERE username = ?"
    ).await?);

    let mut pull = zeromq::PullSocket::new();
    pull.bind("tcp://0.0.0.0:5555").await?;

    let mut push = zeromq::PushSocket::new();
    push.connect("tcp://0.0.0.0:5556").await?;

    let arc_scylla = Arc::new(scylla);
    let arc_push = Arc::new(Mutex::new(push));

    println!("Ready");

    loop {
        let res: String = pull.recv().await?.try_into()?;

        let threaded_scylla = arc_scylla.clone();
        let threaded_query = arc_query.clone();
        let threaded_push = arc_push.clone();

        tokio::spawn(async move {
            let (id, user) = res.split_at(21);

            let scylla = threaded_scylla;
            let query = threaded_query;

            let execution = scylla.execute(
                &query, 
                (user,)
            ).await.expect("Something went wrong").rows;

            match execution {
                None => {
                    let mut push = threaded_push.lock().await;

                    push.send("".into()).await.expect("Something went wrong");
                }
                Some(rows) => {
                    match rows.into_typed::<(String, String)>().next() {
                        None => {
                            let mut push = threaded_push.lock().await;

                            push.send(id.into()).await.expect("Something went wrong");
                        }
                        Some(row) => {
                            let (username, password) = row.expect("Something went wrong");
                            let concat = username + " " + &password;

                            let mut push = threaded_push.lock().await;

                            push.send((id.to_owned() + &concat)
                                .try_into()
                                .expect("Something went wrong")
                            )
                                .await
                                .expect("Something went wrong");
                        }
                    }
                }
            };
        });
    }
}
