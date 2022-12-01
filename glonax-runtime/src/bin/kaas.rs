use glonax::net::ControlNet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let net = ControlNet::new("can0", 0x9b)?;

    net.request(0x20, PGN::AddressClaimed).await;

    // let frame = FrameBuilder::new(IdBuilder::from_pgn(61184).da(0x20).sa(0x9b).build())
    //     .copy_from_slice(&[0x00, 0xee, 0xff])
    //     .build();

    let op = u16::to_le_bytes(0x04);
    println!("op {:02X?}", op);

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
