use std::{
    collections::HashSet,
    sync::{atomic, Arc, Mutex},
    thread,
    time::Duration,
};

use tokio::prelude::*;
use tokio::stream::StreamExt;
use tokio::sync::mpsc::channel;
use tokio::time;
use uuid::Uuid;

use bluster::{
    gatt::{
        characteristic,
        characteristic::Characteristic,
        descriptor,
        descriptor::Descriptor,
        event::{Event, Response},
        service::Service,
    },
    Peripheral, SdpShortUuid,
};
use std::sync::atomic::{AtomicBool, Ordering};

const ADVERTISING_NAME: &str = "hello";
const ADVERTISING_TIMEOUT: u64 = 60;

#[tokio::test]
async fn it_advertises_gatt() {
    env_logger::init();

    let (sender_characteristic, mut receiver_characteristic) = channel(1);
    let (sender_descriptor, mut receiver_descriptor) = channel(1);

    // characteristics
    let mut characteristics: HashSet<Characteristic> = HashSet::new();
    characteristics.insert(Characteristic::new(
        Uuid::from_sdp_short_uuid(0x2A3D as u16),
        characteristic::Properties::new(
            Some(characteristic::Read(characteristic::Secure::Insecure(
                sender_characteristic.clone(),
            ))),
            Some(characteristic::Write::WithResponse(
                characteristic::Secure::Insecure(sender_characteristic.clone()),
            )),
            Some(sender_characteristic.clone()),
            None,
        ),
        None,
        {
            let mut descriptors = HashSet::<Descriptor>::new();
            descriptors.insert(Descriptor::new(
                Uuid::from_sdp_short_uuid(0x2A3D as u16),
                descriptor::Properties::new(
                    Some(descriptor::Read(descriptor::Secure::Insecure(
                        sender_descriptor.clone(),
                    ))),
                    Some(descriptor::Write(descriptor::Secure::Insecure(
                        sender_descriptor.clone(),
                    ))),
                ),
                None,
            ));
            descriptors
        },
    ));

    // create peripheral
    let peripheral = Peripheral::new().await.unwrap();
    peripheral
        .add_service(&Service::new(
            Uuid::from_sdp_short_uuid(0x1234 as u16),
            true,
            characteristics,
        ))
        .unwrap();

    // wait for power on
    loop {
        let is_powered = peripheral.is_powered().await.unwrap();
        if is_powered {
            println!("Peripheral powered on");
            break;
        }
        println!("Wait for peripheral powered on");
        time::delay_for(Duration::from_millis(500)).await;
    }

    // start advertising
    let advertising_stream = peripheral
        .start_advertising(ADVERTISING_NAME, &[])
        .await
        .unwrap();
    let advertising_status = AtomicBool::new(true);
    // tokio::spawn(advertising_stream.for_each(|_| {
    //     if advertising_status.load(Ordering::Relaxed) {
    //         println!("Peripheral started advertising \"{}\"", ADVERTISING_NAME);
    //     }
    // }));

    let characteristic_value = Arc::new(Mutex::new(String::from("hi")));
    let notifying = Arc::new(atomic::AtomicBool::new(false));
    let descriptor_value = Arc::new(Mutex::new(String::from("hi")));

    // receiving characteristic request
    tokio::spawn(async move {
        while let Some(event) = receiver_characteristic.recv().await {
            match event {
                Event::ReadRequest(read_request) => {
                    println!(
                        "GATT server got a read request with offset {}!",
                        read_request.offset
                    );
                    let value = characteristic_value.lock().unwrap().clone();
                    read_request
                        .response
                        .send(Response::Success(value.clone().into()))
                        .unwrap();
                    println!("GATT server responded with \"{}\"", value);
                }
                Event::WriteRequest(write_request) => {
                    let new_value = String::from_utf8(write_request.data).unwrap();
                    println!(
                        "GATT server got a write request with offset {} and data {}!",
                        write_request.offset, new_value,
                    );
                    *characteristic_value.lock().unwrap() = new_value;
                    write_request
                        .response
                        .send(Response::Success(vec![]))
                        .unwrap();
                }
                Event::NotifySubscribe(notify_subscribe) => {
                    println!("GATT server got a notify subscription!");
                    let notifying = Arc::clone(&notifying);
                    notifying.store(true, atomic::Ordering::Relaxed);
                    thread::spawn(move || {
                        let mut count = 0;
                        loop {
                            if !(&notifying).load(atomic::Ordering::Relaxed) {
                                break;
                            };
                            count += 1;
                            println!("GATT server notifying \"hi {}\"!", count);
                            notify_subscribe
                                .clone()
                                .notification
                                .try_send(format!("hi {}", count).into())
                                .unwrap();
                            thread::sleep(Duration::from_secs(2));
                        }
                    });
                }
                Event::NotifyUnsubscribe => {
                    println!("GATT server got a notify unsubscribe!");
                    notifying.store(false, atomic::Ordering::Relaxed);
                }
            };
        }
    });

    // receiving descriptor request
    tokio::spawn(async move {
        while let Some(event) = receiver_descriptor.recv().await {
            match event {
                Event::ReadRequest(read_request) => {
                    println!(
                        "GATT server got a read request with offset {}!",
                        read_request.offset
                    );
                    let value = descriptor_value.lock().unwrap().clone();
                    read_request
                        .response
                        .send(Response::Success(value.clone().into()))
                        .unwrap();
                    println!("GATT server responded with \"{}\"", value);
                }
                Event::WriteRequest(write_request) => {
                    let new_value = String::from_utf8(write_request.data).unwrap();
                    println!(
                        "GATT server got a write request with offset {} and data {}!",
                        write_request.offset, new_value,
                    );
                    *descriptor_value.lock().unwrap() = new_value;
                    write_request
                        .response
                        .send(Response::Success(vec![]))
                        .unwrap();
                }
                _ => eprintln!("Event not supported for Descriptors!"),
            };
        }
    });

    // stop advertising
    time::delay_for(Duration::from_secs(30)).await;
    peripheral.stop_advertising().await.unwrap();
    while let is_advertising = peripheral.is_advertising().await.unwrap() {
        if !is_advertising {
            println!("Peripheral stopped advertising");
            break;
        }
        time::delay_for(Duration::from_millis(100)).await;
    }
}
