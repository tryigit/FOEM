use std::time::Instant;

fn mock_adb_shell_slow(_serial: &str, args: &[&str]) -> Result<String, String> {
    std::thread::sleep(std::time::Duration::from_millis(100)); // Simulate ADB overhead
    if args == ["getprop", "gsm.sim.operator.alpha"] {
        Ok("SomeOperator".to_string())
    } else if args == ["getprop", "gsm.sim.operator.numeric"] {
        Ok("12345".to_string())
    } else if args == ["getprop", "gsm.sim.state"] {
        Ok("READY".to_string())
    } else if args == ["getprop", "gsm.network.type"] {
        Ok("LTE".to_string())
    } else if args == ["getprop", "gsm.current.phone-type"] {
        Ok("GSM".to_string())
    } else {
        Err("".to_string())
    }
}

pub fn check_carrier_lock_slow(serial: &str) -> String {
    let props = [
        ("Operator", "gsm.sim.operator.alpha"),
        ("Operator Code", "gsm.sim.operator.numeric"),
        ("SIM State", "gsm.sim.state"),
        ("Network Type", "gsm.network.type"),
        ("Phone Type", "gsm.current.phone-type"),
    ];
    let mut output = String::from("Carrier/SIM Status:\n");
    for (label, prop) in &props {
        match mock_adb_shell_slow(serial, &["getprop", prop]) {
            Ok(val) if !val.is_empty() => output.push_str(&format!("  {}: {}\n", label, val)),
            _ => output.push_str(&format!("  {}: --\n", label)),
        }
    }
    output
}

fn mock_adb_shell_fast(_serial: &str, args: &[&str]) -> Result<String, String> {
    std::thread::sleep(std::time::Duration::from_millis(100)); // Simulate one ADB overhead
    if args.len() == 3 && args[0] == "sh" && args[1] == "-c" {
        // Mocked batched response
        Ok("SomeOperator\nB_MARKER\n12345\nB_MARKER\nREADY\nB_MARKER\nLTE\nB_MARKER\nGSM\nB_MARKER\n".to_string())
    } else {
        Err("".to_string())
    }
}

pub fn check_carrier_lock_fast(serial: &str) -> String {
    let props = [
        ("Operator", "gsm.sim.operator.alpha"),
        ("Operator Code", "gsm.sim.operator.numeric"),
        ("SIM State", "gsm.sim.state"),
        ("Network Type", "gsm.network.type"),
        ("Phone Type", "gsm.current.phone-type"),
    ];

    // Instead of N calls, we do 1 call to shell with a script
    // that echoes all properties separated by a marker
    let mut script = String::new();
    for (_, prop) in &props {
        script.push_str(&format!("getprop {}; echo B_MARKER; ", prop));
    }

    let mut output = String::from("Carrier/SIM Status:\n");
    match mock_adb_shell_fast(serial, &["sh", "-c", &script]) {
        Ok(res) => {
            let mut parts = res.split("B_MARKER");
            for (label, _) in &props {
                let val = parts.next().unwrap_or("").trim();
                if !val.is_empty() {
                    output.push_str(&format!("  {}: {}\n", label, val));
                } else {
                    output.push_str(&format!("  {}: --\n", label));
                }
            }
        }
        Err(_) => {
            for (label, _) in &props {
                output.push_str(&format!("  {}: --\n", label));
            }
        }
    }

    output
}

fn main() {
    println!("Running slow version...");
    let start_slow = Instant::now();
    let res_slow = check_carrier_lock_slow("serial");
    let dur_slow = start_slow.elapsed();
    println!("Slow version took: {:?}", dur_slow);

    println!("\nRunning fast version...");
    let start_fast = Instant::now();
    let res_fast = check_carrier_lock_fast("serial");
    let dur_fast = start_fast.elapsed();
    println!("Fast version took: {:?}", dur_fast);

    assert_eq!(res_slow, res_fast);
    println!("\nOutputs match!");
}
