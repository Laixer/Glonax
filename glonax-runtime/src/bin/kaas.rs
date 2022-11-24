use glonax::net::ControlNet;
use glonax_j1939::{Frame, FrameBuilder, IdBuilder};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let net = ControlNet::new("can0", 0x9b)?;

    let stream = net.stream();

    // {
    //     let frame = FrameBuilder::new(IdBuilder::from_pgn(59_904).da(0x20).sa(0x9b).build())
    //         .copy_from_slice(&[0x00, 0xee, 0x00])
    //         .build();

    //     stream.write(&frame).await.unwrap();
    //     println!("{}", frame);
    // }

    pub fn request_for_message(pgn: u16, da: u8, sa: u8) -> Frame {
        let byte_array = u32::to_be_bytes(pgn as u32);

        FrameBuilder::new(IdBuilder::from_pgn(59_904).da(da).sa(sa).build())
            .copy_from_slice(&[byte_array[3], byte_array[2], byte_array[1]])
            .build()
    }

    {
        let frame = request_for_message(0xee00, 0x20, 0x9b);

        stream.write(&frame).await.unwrap();
        println!("{}", frame);
    }

    // let frame = FrameBuilder::new(IdBuilder::from_pgn(61184).da(0x20).sa(0x9b).build())
    //     .copy_from_slice(&[0x00, 0xee, 0xff])
    //     .build();

    let op = u16::to_le_bytes(0x04);
    println!("op {:X?}", op);

    let mur = u32::to_le_bytes(0x4001);
    println!("mur {:X?}", mur);

    let tmr = u32::to_le_bytes(0x10000000);
    println!("tmr {:X?}", tmr);

    let cyl = u32::to_le_bytes(0x32);
    println!("cyl {:X?}", cyl);

    let term = 0x1;
    println!("term {:X?}", term);

    let pre_val = u32::to_le_bytes(0x00000000);
    println!("pre_val {:X?}", pre_val);

    let pre_en = 0x00;
    println!("pre_en {:X?}", pre_en);

    let baud = 0x01;
    println!("baud {:X?}", baud);

    // 04 00
    // 00 40 00 00
    // 00 00 00 10
    // 32 00 00 00
    // 01
    // 00 00 00 00
    // 00
    // 01

    // TX

    // 04 00
    // 01 40 00 00
    // 00 00 00 10
    // 32 00 00 00
    // FF
    // 00 00 00 00
    // 00
    // FF

    // 01 04 00 01 40 00 00 00
    // 02 00 00 10 32 00 00 00
    // 03 FF 00 00 00 00 00 FF

    // RX
    // cansend can0 18EA2010#00EF00
    // sleep 0.1
    // cansend can0 18EC2010#110301FFFF00EF00
    // sleep 0.1
    // cansend can0 18EC2010#13150003FF00EF00
    
    // TX
    // cansend can0 18EC2010#10150003FF00EF00
    // sleep 0.1
    // cansend can0 18EB2010#0104000040000000
    // cansend can0 18EB2010#0200001032000000
    // cansend can0 18EB2010#03000000000000FF
    

    Ok(())
}