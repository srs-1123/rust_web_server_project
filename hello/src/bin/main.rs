extern crate hello;
use hello::ThreadPool;

use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::fs::File;
use std::time::Duration;
use std::thread;

fn main() {
    // unwrapでResultの中身を取り出す
    // Ok(TcpLister)なら、その中身を返す
    // Errならプログラムをパニックさせて終了
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // 4つのワーカースレッドを持つスレッドプールを作成
    let pool = ThreadPool::new(4);

    // listener.incoming(): 新しい接続を待つイテレータを返す
    // take(2): 最初の2つの接続のみを処理する制限
    // リクエストを待ち受けている間はincomingがブロッキング状態になる
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        // streamの所有権はhandle_connectionにうつる
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let sleep = b"GET /sleep HTTP/1.1\r\n";
    let get = b"GET / HTTP/1.1\r\n";
    
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let mut file = File::open(filename).unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let response = format!("{}{}", status_line, contents); // format!: sprintfみたいな機能

    stream.write(response.as_bytes()).unwrap(); // 文字列をバイトに変換
    stream.flush().unwrap(); // 待機し、 バイトが全て接続に書き込まれるまでプログラムが継続するのを防ぐ
}