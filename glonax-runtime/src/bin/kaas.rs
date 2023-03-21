use glonax::net::J1939Network;
use glonax_j1939::PGN;

#[tokio::main]
async fn main() {
    let net = J1939Network::new("can0", 0x9b).expect("ouch");
    net.set_promisc_mode(true).unwrap();

    // net.request(0x20, PGN::AddressClaimed).await;
    loop {
        let frame = net.accept().await.unwrap();

        println!("{}", frame);
    }

    // net.broadcast(65_240, &[0xff; 9]).await;

    // let frame = FrameBuilder::new(IdBuilder::from_pgn(61184).da(0x20).sa(0x9b).build())
    //     .copy_from_slice(&[0x00, 0xee, 0xff])
    //     .build();

    // let op = u16::to_le_bytes(0x04);
    // println!("op {:02X?}", op);

    // let mur = u32::to_le_bytes(0x4001);
    // println!("mur {:X?}", mur);

    // let tmr = u32::to_le_bytes(0x10000000);
    // println!("tmr {:X?}", tmr);

    // let cyl = u32::to_le_bytes(0x32);
    // println!("cyl {:X?}", cyl);

    // let term = 0x1;
    // println!("term {:X?}", term);

    // let pre_val = u32::to_le_bytes(0x00000000);
    // println!("pre_val {:X?}", pre_val);

    // let pre_en = 0x00;
    // println!("pre_en {:X?}", pre_en);

    // let baud = 0x01;
    // println!("baud {:X?}", baud);

    // OperatingParameter:      04 00
    // MUR:                     00 40 00 00
    // TMR:                     00 00 00 10
    // SensorCycleTime:         32 00 00 00
    // CANBusTermination:       01
    // SensorPresetValue:       00 00 00 00
    // SensorPresetEnable:      00
    // BaudRate:                01

    // Get configuration
    // RX Receiving via transport
    // DATA: config (21)
    //
    // cansend can0 18EA 20 10 # 00EF00
    // sleep 0.1
    // cansend can0 18EC 20 10 # 110301FFFF00EF00
    // sleep 0.1
    // cansend can0 18EC 20 10 # 13150003FF00EF00

    // Set configuration
    //
    // TX Sending via transport
    // DATA: config (21)
    //
    // cansend can0 18EC 20 10#10150003FF00EF00
    // sleep 0.1
    // cansend can0 18EB 20 10 # 01 04 00 FF 3F 00 00 FF
    // cansend can0 18EB 20 10 # 02 3F 00 00 32 00 00 00
    // cansend can0 18EB 20 10 # 03 00 00 00 00 00 00 FF

    // Change node ID
    //
    // TX Sending via BAM
    // Data: NAME (8) + ID (1)
    //
    // cansend can0 18EC 20 10 # 20 09 00 02 FF D8 FE 00
    // sleep 0.1
    // cansend can0 18EB 20 10 # 01 18 A4 49 24 11 05 06
    // cansend can0 18EB 20 10 # 02 85 6B FF FF FF FF FF

    // cansend can0 18EA6B10#00EF00
    // sleep 0.1
    // cansend can0 18EC6B10#110301FFFF00EF00
    // sleep 0.1
    // cansend can0 18EC6B10#13150003FF00EF00

    // cansend can0 18EC6B10#10150003FF00EF00
    // sleep 0.1
    // cansend can0 18EB6B10#010500FF3F0000B8
    // cansend can0 18EB6B10#020B000032000000
    // cansend can0 18EB6B10#03FF0000000001FF

    // 05, 00
    // FF, 3F, 00, 00
    // B8  0B, 00, 00
    // 32, 00, 00, 00
    // 00
    // 00, 00, 00, 00
    // 00
    // 01

    // // 6283

    // cansend can0 18EC6B10#10150003FF00EF00
    // sleep 0.1
    // cansend can0 18EB6B10#0105008B1800008B
    // cansend can0 18EB6B10#0218000032000000
    // cansend can0 18EB6B10#03FF0000000001FF

    // 0000188B
    // 8B180000

// cansend can0 18EA6B10#00EF00
// sleep 0.1
// cansend can0 18EC6B10#110301FFFF00EF00
// sleep 0.1
// cansend can0 18EC6B10#13150003FF00EF00
}
