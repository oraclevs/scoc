use scoc::{OptionValue, ParseOptions};

#[test]
fn ping_batch_parses_linux_reply_and_summary() {
    let input = b"PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.\n64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=10.0 ms\n\n--- 1.1.1.1 ping statistics ---\n1 packets transmitted, 1 received, 0% packet loss, time 0ms\nrtt min/avg/max/mdev = 10.000/10.000/10.000/0.000 ms\n";
    let value = scoc::parse("ping", input, &Default::default()).unwrap();
    assert_eq!(value["destination_ip"], "1.1.1.1");
    assert_eq!(value["packets_transmitted"], 1);
    assert_eq!(value["responses"][0]["type"], "reply");
    assert_eq!(value["responses"][0]["timestamp"], serde_json::Value::Null);
    assert_eq!(value["responses"][0]["time_ms"], 10.0);
    assert_eq!(value["time_ms"], 0);
    assert_eq!(value["responses"][0]["bytes"], 64);
    assert!(value["responses"][0].get("response_bytes").is_none());
    assert!(value["responses"][0].get("sent_bytes").is_none());
}

#[test]
fn ping_stream_waits_for_complete_line_across_chunks() {
    let mut parser = scoc::stream_parser("ping", &Default::default()).unwrap();
    assert!(parser.push(b"PING 1.1.1.1 (1.1.").unwrap().is_empty());
    assert!(parser
        .push(b"1.1) 56(84) bytes of data.\n64 bytes fr")
        .unwrap()
        .is_empty());
    let rows = parser
        .push(b"om 1.1.1.1: icmp_seq=1 ttl=57 time=10.0 ms\n")
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["type"], "reply");
    assert_eq!(rows[0]["destination_ip"], "1.1.1.1");
}

#[test]
fn two_ping_streams_do_not_share_state() {
    let mut a = scoc::stream_parser("ping", &Default::default()).unwrap();
    let mut b = scoc::stream_parser("ping", &Default::default()).unwrap();
    a.push(b"PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.\n")
        .unwrap();
    b.push(b"PING 8.8.8.8 (8.8.8.8) 56(84) bytes of data.\n")
        .unwrap();
    let ar = a
        .push(b"64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=1.0 ms\n")
        .unwrap();
    let br = b
        .push(b"64 bytes from 8.8.8.8: icmp_seq=1 ttl=57 time=2.0 ms\n")
        .unwrap();
    assert_eq!(ar[0]["destination_ip"], "1.1.1.1");
    assert_eq!(br[0]["destination_ip"], "8.8.8.8");
}

#[test]
fn ping_stream_ignore_errors_emits_jc_meta() {
    let options = ParseOptions::from_pairs([("ignoreErrors", OptionValue::Bool(true))]);
    let mut parser = scoc::stream_parser("ping", &options).unwrap();
    parser
        .push(b"PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.\n")
        .unwrap();
    let rows = parser.push(b"this is not ping output\n").unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["_jc_meta"]["success"], false);
}

#[test]
fn ping_stream_ignore_errors_marks_successful_records_too() {
    let options = ParseOptions::from_pairs([("ignoreErrors", OptionValue::Bool(true))]);
    let mut parser = scoc::stream_parser("ping", &options).unwrap();
    parser
        .push(b"PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.\n")
        .unwrap();
    let rows = parser
        .push(b"64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=10.0 ms\n")
        .unwrap();
    assert_eq!(rows[0]["_jc_meta"]["success"], true);
}

#[test]
fn ping_batch_preserves_jc_response_key_order() {
    let input = b"PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.\n64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=10.0 ms\n\n--- 1.1.1.1 ping statistics ---\n1 packets transmitted, 1 received, 0% packet loss, time 0ms\nrtt min/avg/max/mdev = 10.000/10.000/10.000/0.000 ms\n";
    let value = scoc::parse("ping", input, &Default::default()).unwrap();
    let keys: Vec<_> = value["responses"][0]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        [
            "type",
            "timestamp",
            "bytes",
            "response_ip",
            "icmp_seq",
            "ttl",
            "time_ms",
            "duplicate"
        ]
    );
}
