use std::net::{TcpStream, Ipv4Addr, SocketAddrV4, Shutdown, SocketAddrV6, Ipv6Addr};
use std::io::{Read, Error, Write};
// use std::convert::TryInto;

const VERSION:u8=0x05;
const CONNECT: u8 = 0x01;
const BIND: u8 = 0x02;
const UDP_ASSOCIATE: u8 = 0x03;



pub struct Socks5{
    stream:TcpStream
}

impl Socks5{
    pub fn new(stream:TcpStream) ->Socks5{
        Socks5{
            stream
        }
    }
    pub fn serve(&mut self){
        //1. 握手
        println!("2  握手");
        self.hand_shake();

        //2. 验证

        //3.获取地址转发数据
        println!("3 准备转发");
        self.ready_serve();

        //4.关闭
        self.stream.shutdown(Shutdown::Both);
        println!("9 双向流关闭");

    }


    //握手
    // version identifier/method selection message
    // +----+----------+----------+
    // |VER | NMETHODS | METHODS  |
    // +----+----------+----------+
    // | 1  |    1     | 1 to 255 |
    // +----+----------+----------+
    // reply:
    // +----+--------+
    // |VER | METHOD |
    // +----+--------+
    // |  1 |   1    |
    // +----+--------+
    // hand_shake dail hand_shake between socks5 client and socks5 server.
    fn hand_shake(&mut self)-> Result<u8,Error> {
        let mut b1:[u8;1]=[0;1];
        self.stream.read(& mut b1);
        if b1[0]!=VERSION{
            return Err(std::io::Error::new(std::io::ErrorKind::Other,"fuck"));
        }
        self.stream.read(& mut b1);
        let nmethods=b1[0] as usize;

        let mut buf = [0u8;257];
        //读取请求的验证方法
        let _=self.stream.read_exact(&mut buf[2..2+nmethods]);

        // reply to socks5 client
        //00 无需验证
        let _=self.stream.write(&[VERSION,0x00]);
        return Ok(b1[0]);
    }

    //交换信息
    //获取要访问的地址和端口，并将客户端的流转发过去
    //再把响应流回传给客户端
    // The SOCKS request is formed as follows:
    //         +----+-----+-------+------+----------+----------+
    //         |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
    //         +----+-----+-------+------+----------+----------+
    //         | 1  |  1  | X’00’ |  1   | Variable |    2     |
    //         +----+-----+-------+------+----------+----------+
    // Where:
    //           o  VER    protocol version: X’05’
    //           o  CMD
    //              o  CONNECT X’01’
    //              o  BIND X’02’
    //              o  UDP ASSOCIATE X’03’
    //           o  RSV    RESERVED
    //           o  ATYP   address type of following address
    //              o  IP V4 address: X’01’
    //              o  DOMAINNAME: X’03’
    //              o  IP V6 address: X’04’
    //           o  DST.ADDR       desired destination address
    //           o  DST.PORT desired destination port in network octet order

    // get_cmd gets the cmd requested by socks5 client.
    //正式处理请求
    fn ready_serve(&mut self){
        let mut buf = [0u8;1];
        self.stream.read(&mut buf);
        let ver = buf[0];
        if ver!=VERSION {
            println!("socks版本不对，结束");
            return;
        }
        self.stream.read(&mut buf);
        let cmd = buf[0];

        match cmd {
            CONNECT|BIND|UDP_ASSOCIATE=>{}
            _=>{
                println!("fuck errrrror");
                return;
            }
        }

        //rsv X00
        self.stream.read(&mut buf);
        //atyp
        self.stream.read(&mut buf);
        let atyp = buf[0];

        let addr_len = match atyp {
            0x1=>{
                //ipv4的话 4个字节
                4 as usize
            }
            0x3=>{
                //domainname的话值的第一个字节代表域名长度->往后走一个字节
                self.stream.read(&mut buf);
                buf[0] as usize
            }
            0x4=>{
                //ipv6 16个字节
                16 as usize
            }
            _=>{
                println!("获取地址失败！");
                return;
            }
        };
        //读取地址
        let mut buf=[0u8;260];
        let _=self.stream.read_exact(&mut buf[0..addr_len]);

        //读取port
        let mut port_buf=[0u8;2];
        self.stream.read(&mut port_buf);
        let port = (port_buf[0] as u16)<<8|port_buf[1] as u16;

        //返回响应
        // returns a reply formed as follows:
        //         +----+-----+-------+------+----------+----------+
        //         |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
        //         +----+-----+-------+------+----------+----------+
        //         | 1  |  1  | X’00’ |  1   | Variable |    2     |
        //         +----+-----+-------+------+----------+----------+
        // Where:
        //           o  VER    protocol version: X’05’
        //           o  REP    Reply field:
        //              o  X’00’ succeeded
        //              o  X’01’ general SOCKS server failure
        //              o  X’02’ connection not allowed by ruleset
        //              o  X’03’ Network unreachable
        //              o  X’04’ Host unreachable
        //              o  X’05’ Connection refused
        //              o  X’06’ TTL expired
        //              o  X’07’ Command not supported
        //              o  X’08’ Address type not supported
        //              o  X’09’ to X’FF’ unassigned
        //           o  RSV    RESERVED
        //           o  ATYP   address type of following address
        //              o  IP V4 address: X’01’
        //              o  DOMAINNAME: X’03’
        //              o  IP V6 address: X’04’
        //           o  BND.ADDR       server bound address
        //           o  BND.PORT       server bound port in network octet order
        self.stream.write(&[VERSION,
            0x00,
            0x00,
            0x1,
            0x00,
            0x00,
            0x00,
            0x00,
            0x10,
            0x10]);

        //开始转发
        println!("4  获取到地址开始转发");
        match atyp {
            0x1=>{
                let ip = Ipv4Addr::new(buf[0],buf[1],buf[2],buf[3]);
                println!("IP is {}:",ip);
                let addr = SocketAddrV4::new(ip,port);
                let mut remote = TcpStream::connect(addr).unwrap();
                self.forward(remote);
            }

            0x3=>{
                println!("domain name is:,{}",String::from_utf8_lossy(&buf[..addr_len]));
                let addr = String::new()+String::from_utf8_lossy(&buf[..addr_len]).trim()+":"+port.to_string().as_str();
                let mut remote = TcpStream::connect(addr.as_str()).unwrap();
                self.forward(remote);
            }
            0x4=>{

                let a = SocketAddrV6::new(Ipv6Addr::new(
                    (buf[0] as u16 )<< 8|buf[1] as u16,
                    (buf[2] as u16 )<< 8|buf[3] as u16,
                    (buf[4] as u16 )<< 8|buf[5] as u16,
                    (buf[6] as u16 )<< 8|buf[7] as u16,
                    (buf[8] as u16 )<< 8|buf[9] as u16,
                    (buf[10] as u16 )<< 8|buf[11] as u16,
                    (buf[12] as u16 )<< 8|buf[13] as u16,
                    (buf[14] as u16 )<< 8|buf[15] as u16
                ),port,0,0);
                println!("0x4,{}",a.ip().to_string());
                let mut remote = TcpStream::connect(a).unwrap();
                self.forward(remote);
            }
            _=>{}
        }
    }

    fn forward(&mut self, mut remote:TcpStream){
        let mut remote_copy = remote.try_clone().unwrap();
        let mut client = self.stream.try_clone().unwrap();

        //一个线程从客户端读取写入服务端
        //主线程将响应数据写回客户端
        std::thread::spawn(move||{
            println!("5 读取C端数据写向S");
            let mut client_buf = [0u8;4096];
            loop {
                match client.read(&mut client_buf) {
                    Ok(n)=>{
                        if n==0 {
                            println!("C端写向S完成0 break");
                            break;
                        }
                        remote_copy.write(&mut client_buf[0..n]);
                    }
                    Err(err)=>{
                        println!("{:?}",err);
                        break;
                    }
                }
            }
            println!("6 C->S lopp结束");
        });

        let mut server_buf = [0u8;4096];
        println!("7 S->C 开始");
        loop {
            match remote.read(&mut server_buf) {
                Ok(n)=>{
                    if n==0 {
                        println!("7.1 读取到0");
                        break;
                    }
                    self.stream.write(&mut server_buf[..n]);
                }
                Err(err)=>{
                    println!("{:?}",err);
                    break;
                }
            }
        }
        println!("8 S->C loop结束");
    }
    //转发
}
