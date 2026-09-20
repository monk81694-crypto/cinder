// slp-probe - minimal Minecraft Java Server List Ping probe
// std only, no dependencies
// Usage: slp-probe <host> [port=25565] [timeout_ms=5000]

use std::env;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::process;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for c in s.chars() {
        let n = c as u32;
        if n == 34 {
            out.push_str("\\\"");
        } else if n == 92 {
            out.push_str("\\\\");
        } else if n == 10 {
            out.push_str("\\n");
        } else if n == 13 {
            out.push_str("\\r");
        } else if n == 9 {
            out.push_str("\\t");
        } else if n == 8 {
            out.push_str("\\b");
        } else if n == 12 {
            out.push_str("\\f");
        } else if n < 32 {
            let hex = "0123456789abcdef";
            let hb = hex.as_bytes();
            out.push_str("\\u00");
            out.push(hb[((n >> 4) & 15) as usize] as char);
            out.push(hb[(n & 15) as usize] as char);
        } else {
            out.push(c);
        }
    }
    out
}

fn strip_format_codes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars();
    loop {
        let oc = it.next();
        if oc.is_none() {
            break;
        }
        let c = oc.unwrap();
        if c as u32 == 167 {
            let _ = it.next();
        } else {
            out.push(c);
        }
    }
    out
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut n = 0usize;
    for c in s.chars() {
        if n >= max_chars {
            break;
        }
        out.push(c);
        n += 1;
    }
    out
}

fn write_varint(buf: &mut Vec<u8>, v: i32) {
    let mut uv = v as u32;
    loop {
        let mut b = (uv & 127) as u8;
        uv >>= 7;
        if uv != 0 {
            b |= 128;
        }
        buf.push(b);
        if uv == 0 {
            break;
        }
    }
}

fn read_varint<R: Read>(r: &mut R) -> std::io::Result<i32> {
    let mut num: u32 = 0;
    let mut shift = 0u32;
    loop {
        if shift >= 35 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "varint too big"));
        }
        let mut tmp = [0u8; 1];
        r.read_exact(&mut tmp)?;
        let byte = tmp[0];
        num |= ((byte & 127) as u32) << shift;
        shift += 7;
        if (byte & 128) == 0 {
            break;
        }
    }
    Ok(num as i32)
}

fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && (b[i] == 32 || b[i] == 10 || b[i] == 13 || b[i] == 9) {
        i += 1;
    }
    i
}

fn parse_json_string_at(s: &str, start: usize) -> Option<(String, usize)> {
    let b = s.as_bytes();
    if start >= b.len() || b[start] != 34 {
        return None;
    }
    let mut out = String::new();
    let mut i = start + 1;
    while i < b.len() {
        let c = b[i];
        if c == 34 {
            return Some((out, i + 1));
        } else if c == 92 {
            i += 1;
            if i >= b.len() {
                return None;
            }
            let e = b[i];
            if e == 34 {
                out.push(34 as char);
            } else if e == 92 {
                out.push(92 as char);
            } else if e == 47 {
                out.push(47 as char);
            } else if e == 98 {
                out.push(8 as char);
            } else if e == 102 {
                out.push(12 as char);
            } else if e == 110 {
                out.push(10 as char);
            } else if e == 114 {
                out.push(13 as char);
            } else if e == 116 {
                out.push(9 as char);
            } else if e == 117 {
                if i + 4 >= b.len() {
                    return None;
                }
                let hexpart = std::str::from_utf8(&b[i + 1..i + 5]).ok()?;
                let mut cp = u32::from_str_radix(hexpart, 16).ok()?;
                i += 4;
                if cp >= 55296 && cp <= 56319 {
                    if i + 6 < b.len() && b[i + 1] == 92 && b[i + 2] == 117 {
                        if let Ok(hex2) = std::str::from_utf8(&b[i + 3..i + 7]) {
                            if let Ok(lo) = u32::from_str_radix(hex2, 16) {
                                if lo >= 56320 && lo <= 57343 {
                                    cp = 65536 + ((cp - 55296) << 10) + (lo - 56320);
                                    i += 6;
                                }
                            }
                        }
                    }
                }
                let ch = char::from_u32(cp).unwrap_or(std::char::REPLACEMENT_CHARACTER);
                out.push(ch);
            } else {
                return None;
            }
            i += 1;
        } else {
            let rest = &s[i..];
            let ch = rest.chars().next()?;
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    None
}

fn find_key_colon(json: &str, key: &str, from: usize) -> Option<usize> {
    let b = json.as_bytes();
    let mut i = from;
    while i < b.len() {
        if b[i] == 34 {
            let parsed = parse_json_string_at(json, i);
            if parsed.is_none() {
                return None;
            }
            let tmp = parsed.unwrap();
            let decoded = tmp.0;
            let nxt = tmp.1;
            if decoded == key {
                let j = skip_ws(b, nxt);
                if j < b.len() && b[j] == 58 {
                    return Some(j);
                }
            }
            i = nxt;
        } else {
            i += 1;
        }
    }
    None
}

fn extract_brace_slice(json: &str, open_pos: usize, open_b: u8, close_b: u8) -> Option<(usize, usize)> {
    let b = json.as_bytes();
    if open_pos >= b.len() || b[open_pos] != open_b {
        return None;
    }
    let mut depth = 0usize;
    let mut i = open_pos;
    let mut in_str = false;
    let mut esc = false;
    while i < b.len() {
        let c = b[i];
        if in_str {
            if esc {
                esc = false;
            } else if c == 92 {
                esc = true;
            } else if c == 34 {
                in_str = false;
            }
        } else {
            if c == 34 {
                in_str = true;
            } else if c == open_b {
                depth += 1;
            } else if c == close_b {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
                if depth == 0 {
                    return Some((open_pos, i + 1));
                }
            }
        }
        i += 1;
    }
    None
}

fn parse_int_at(json: &str, pos: usize) -> Option<(i64, usize)> {
    let b = json.as_bytes();
    let mut i = skip_ws(b, pos);
    if i >= b.len() {
        return None;
    }
    let mut neg = false;
    if b[i] == 45 {
        neg = true;
        i += 1;
        if i >= b.len() {
            return None;
        }
    }
    if !b[i].is_ascii_digit() {
        return None;
    }
    let mut val: i64 = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        let d = (b[i] - 48) as i64;
        val = val.checked_mul(10)?.checked_add(d)?;
        i += 1;
    }
    if neg {
        val = 0 - val;
    }
    Some((val, i))
}

fn flatten_extra_array(arr_json: &str) -> String {
    let mut out = String::new();
    let b = arr_json.as_bytes();
    let mut i = 1usize;
    loop {
        i = skip_ws(b, i);
        if i >= b.len() {
            break;
        }
        if b[i] == 93 {
            break;
        }
        if b[i] == 44 {
            i += 1;
        } else if b[i] == 34 {
            let parsed = parse_json_string_at(arr_json, i);
            if parsed.is_none() {
                break;
            }
            let tmp = parsed.unwrap();
            out.push_str(tmp.0.as_str());
            i = tmp.1;
        } else if b[i] == 123 {
            let sl = extract_brace_slice(arr_json, i, 123, 125);
            if sl.is_none() {
                break;
            }
            let tmp = sl.unwrap();
            let obj = &arr_json[tmp.0..tmp.1];
            let colon = find_key_colon(obj, "text", 0);
            if colon.is_some() {
                let cpos = colon.unwrap();
                let vs = skip_ws(obj.as_bytes(), cpos + 1);
                if vs < obj.len() && obj.as_bytes()[vs] == 34 {
                    if let Some(pair) = parse_json_string_at(obj, vs) {
                        out.push_str(pair.0.as_str());
                    }
                }
            }
            i = tmp.1;
        } else if b[i] == 91 {
            let sl = extract_brace_slice(arr_json, i, 91, 93);
            if sl.is_none() {
                break;
            }
            let tmp = sl.unwrap();
            let inner = flatten_extra_array(&arr_json[tmp.0..tmp.1]);
            out.push_str(inner.as_str());
            i = tmp.1;
        } else {
            i += 1;
            let mut j = i;
            while j < b.len() && b[j] != 44 && b[j] != 93 {
                j += 1;
            }
            i = j;
        }
    }
    out
}

fn parse_description_value(json: &str, val_pos: usize) -> String {
    let b = json.as_bytes();
    let i = skip_ws(b, val_pos);
    if i >= b.len() {
        return String::new();
    }
    if b[i] == 34 {
        if let Some(pair) = parse_json_string_at(json, i) {
            return pair.0;
        }
        return String::new();
    } else if b[i] == 123 {
        let sl = extract_brace_slice(json, i, 123, 125);
        if sl.is_none() {
            return String::new();
        }
        let tmp = sl.unwrap();
        let obj = &json[tmp.0..tmp.1];
        let mut out = String::new();
        if let Some(cpos) = find_key_colon(obj, "text", 0) {
            let vs = skip_ws(obj.as_bytes(), cpos + 1);
            if vs < obj.len() && obj.as_bytes()[vs] == 34 {
                if let Some(pair) = parse_json_string_at(obj, vs) {
                    out.push_str(pair.0.as_str());
                }
            }
        }
        if let Some(cpos) = find_key_colon(obj, "extra", 0) {
            let vs = skip_ws(obj.as_bytes(), cpos + 1);
            if vs < obj.len() && obj.as_bytes()[vs] == 91 {
                if let Some(se) = extract_brace_slice(obj, vs, 91, 93) {
                    let inner = flatten_extra_array(&obj[se.0..se.1]);
                    out.push_str(inner.as_str());
                }
            }
        }
        return out;
    } else if b[i] == 91 {
        if let Some(se) = extract_brace_slice(json, i, 91, 93) {
            return flatten_extra_array(&json[se.0..se.1]);
        }
        return String::new();
    }
    String::new()
}

fn io_err_msg(e: &std::io::Error) -> String {
    let k = e.kind();
    if k == std::io::ErrorKind::ConnectionRefused {
        return "connection refused".to_string();
    }
    if k == std::io::ErrorKind::TimedOut || k == std::io::ErrorKind::WouldBlock {
        return "timed out".to_string();
    }
    if k == std::io::ErrorKind::HostUnreachable {
        return "host unreachable".to_string();
    }
    if k == std::io::ErrorKind::NetworkUnreachable {
        return "network unreachable".to_string();
    }
    if k == std::io::ErrorKind::ConnectionReset {
        return "connection reset".to_string();
    }
    if k == std::io::ErrorKind::ConnectionAborted {
        return "connection aborted".to_string();
    }
    if k == std::io::ErrorKind::AddrNotAvailable {
        return "address not available".to_string();
    }
    if k == std::io::ErrorKind::UnexpectedEof {
        return "connection closed".to_string();
    }
    let mut s = e.to_string();
    s = s.replace("\n", " ").replace("\r", " ").replace("\t", " ");
    let t = s.trim().to_string().to_lowercase();
    if t.is_empty() {
        return "connect failed".to_string();
    }
    truncate_chars(t.as_str(), 120)
}

fn now_ms() -> u64 {
    let d = SystemTime::now().duration_since(UNIX_EPOCH);
    if d.is_err() {
        return 0;
    }
    d.unwrap().as_millis() as u64
}

fn print_success(players: i64, max: i64, motd: &str, ping_ms: i64, version: &str) {
    let me = json_escape(motd);
    let ve = json_escape(version);
    println!("{{\"ok\":true,\"players\":{},\"max\":{},\"motd\":\"{}\",\"pingMs\":{},\"version\":\"{}\"}}", players, max, me, ping_ms, ve);
}

fn print_failure(msg: &str) {
    let e = truncate_chars(msg, 120);
    let ee = json_escape(e.as_str());
    println!("{{\"ok\":false,\"error\":\"{}\"}}", ee);
}

fn print_usage() {
    println!("{{\"ok\":false,\"error\":\"usage\"}}");
}

fn do_ping(stream: &mut TcpStream) -> Result<i64, String> {
    let payload = now_ms();
    let pb = payload.to_be_bytes();
    let mut pkt: Vec<u8> = Vec::new();
    write_varint(&mut pkt, 9);
    write_varint(&mut pkt, 1);
    pkt.extend_from_slice(&pb);
    let start = Instant::now();
    stream.write_all(pkt.as_slice()).map_err(|e| io_err_msg(&e))?;
    let rl = read_varint(stream).map_err(|e| io_err_msg(&e))?;
    if rl != 9 {
        return Err("invalid response".to_string());
    }
    let rid = read_varint(stream).map_err(|e| io_err_msg(&e))?;
    if rid != 1 {
        return Err("invalid response".to_string());
    }
    let mut back = [0u8; 8];
    stream.read_exact(&mut back).map_err(|e| io_err_msg(&e))?;
    let elapsed = start.elapsed().as_millis() as i64;
    Ok(elapsed)
}

fn run_probe(host: &str, port: u16, timeout_ms: u64, deadline: Instant) -> Result<(i64, i64, String, String, i64), String> {
    if Instant::now() >= deadline {
        return Err("timed out".to_string());
    }
    let op_to = Duration::from_millis(timeout_ms);
    let addr_str = format!("{}:{}", host, port);
    let addrs_res: std::io::Result<Vec<std::net::SocketAddr>> = (host, port).to_socket_addrs().map(|it| it.collect());
    if addrs_res.is_err() {
        let _ = addr_str;
        return Err("dns lookup failed".to_string());
    }
    let addrs = addrs_res.unwrap();
    if addrs.is_empty() {
        return Err("dns lookup failed".to_string());
    }
    let mut last_err = "connect failed".to_string();
    let mut stream_opt: Option<TcpStream> = None;
    for a in addrs {
        if Instant::now() >= deadline {
            return Err("timed out".to_string());
        }
        let c = TcpStream::connect_timeout(&a, op_to);
        if c.is_ok() {
            stream_opt = Some(c.unwrap());
            break;
        } else {
            last_err = io_err_msg(&c.err().unwrap());
        }
    }
    if stream_opt.is_none() {
        return Err(last_err);
    }
    let mut stream = stream_opt.unwrap();
    let _ = stream.set_read_timeout(Some(op_to));
    let _ = stream.set_write_timeout(Some(op_to));
    if Instant::now() >= deadline {
        return Err("timed out".to_string());
    }
    let hb = host.as_bytes();
    let mut hs_payload: Vec<u8> = Vec::new();
    write_varint(&mut hs_payload, 0);
    write_varint(&mut hs_payload, 767);
    write_varint(&mut hs_payload, hb.len() as i32);
    hs_payload.extend_from_slice(hb);
    hs_payload.push((port >> 8) as u8);
    hs_payload.push((port & 255) as u8);
    write_varint(&mut hs_payload, 1);
    let mut hs_packet: Vec<u8> = Vec::new();
    write_varint(&mut hs_packet, hs_payload.len() as i32);
    hs_packet.extend_from_slice(hs_payload.as_slice());
    stream.write_all(hs_packet.as_slice()).map_err(|e| io_err_msg(&e))?;
    let mut req: Vec<u8> = Vec::new();
    write_varint(&mut req, 1);
    write_varint(&mut req, 0);
    stream.write_all(req.as_slice()).map_err(|e| io_err_msg(&e))?;
    let pkt_len = read_varint(&mut stream).map_err(|e| {
        if e.kind() == std::io::ErrorKind::InvalidData {
            return "invalid response".to_string();
        }
        io_err_msg(&e)
    })?;
    if pkt_len <= 0 || pkt_len > 1000000 {
        return Err("invalid response".to_string());
    }
    let pkt_id = read_varint(&mut stream).map_err(|e| {
        if e.kind() == std::io::ErrorKind::InvalidData {
            return "invalid response".to_string();
        }
        io_err_msg(&e)
    })?;
    if pkt_id != 0 {
        return Err("invalid response".to_string());
    }
    let json_len = read_varint(&mut stream).map_err(|e| {
        if e.kind() == std::io::ErrorKind::InvalidData {
            return "invalid response".to_string();
        }
        io_err_msg(&e)
    })?;
    if json_len < 0 || json_len > 1000000 {
        return Err("invalid response".to_string());
    }
    let mut jbuf = vec![0u8; json_len as usize];
    stream.read_exact(jbuf.as_mut_slice()).map_err(|e| io_err_msg(&e))?;
    let json = String::from_utf8(jbuf).map_err(|_| "invalid response".to_string())?;
    let pcolon = find_key_colon(json.as_str(), "players", 0).ok_or("invalid response".to_string())?;
    let pvpos = skip_ws(json.as_bytes(), pcolon + 1);
    if pvpos >= json.len() || json.as_bytes()[pvpos] != 123 {
        return Err("invalid response".to_string());
    }
    let ppair = extract_brace_slice(json.as_str(), pvpos, 123, 125).ok_or("invalid response".to_string())?;
    let psub = &json[ppair.0..ppair.1];
    let ocolon = find_key_colon(psub, "online", 0).ok_or("invalid response".to_string())?;
    let opos = skip_ws(psub.as_bytes(), ocolon + 1);
    let online = parse_int_at(psub, opos).map(|t| t.0).ok_or("invalid response".to_string())?;
    let mcolon = find_key_colon(psub, "max", 0).ok_or("invalid response".to_string())?;
    let mpos = skip_ws(psub.as_bytes(), mcolon + 1);
    let maxv = parse_int_at(psub, mpos).map(|t| t.0).ok_or("invalid response".to_string())?;
    let vcolon = find_key_colon(json.as_str(), "version", 0).ok_or("invalid response".to_string())?;
    let vvpos = skip_ws(json.as_bytes(), vcolon + 1);
    if vvpos >= json.len() || json.as_bytes()[vvpos] != 123 {
        return Err("invalid response".to_string());
    }
    let vpair = extract_brace_slice(json.as_str(), vvpos, 123, 125).ok_or("invalid response".to_string())?;
    let vsub = &json[vpair.0..vpair.1];
    let ncolon = find_key_colon(vsub, "name", 0).ok_or("invalid response".to_string())?;
    let npos = skip_ws(vsub.as_bytes(), ncolon + 1);
    if npos >= vsub.len() || vsub.as_bytes()[npos] != 34 {
        return Err("invalid response".to_string());
    }
    let vname = parse_json_string_at(vsub, npos).map(|t| t.0).ok_or("invalid response".to_string())?;
    let mut raw_motd = String::new();
    if let Some(dcolon) = find_key_colon(json.as_str(), "description", 0) {
        let dpos = skip_ws(json.as_bytes(), dcolon + 1);
        raw_motd = parse_description_value(json.as_str(), dpos);
    }
    let stripped = strip_format_codes(raw_motd.as_str());
    let motd = truncate_chars(stripped.as_str(), 120);
    let ping_ms = do_ping(&mut stream).unwrap_or(-1);
    Ok((online, maxv, motd, vname, ping_ms))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 4 {
        print_usage();
        process::exit(2);
    }
    let host_raw = args[1].clone();
    let host = host_raw.trim().to_string();
    if host.is_empty() {
        print_usage();
        process::exit(2);
    }
    let mut port: u16 = 25565;
    if args.len() >= 3 {
        let pp = args[2].parse::<u16>();
        if pp.is_err() {
            print_usage();
            process::exit(2);
        }
        let pv = pp.unwrap();
        if pv == 0 {
            print_usage();
            process::exit(2);
        }
        port = pv;
    }
    let mut timeout_ms: u64 = 5000;
    if args.len() >= 4 {
        let tp = args[3].parse::<u64>();
        if tp.is_err() {
            print_usage();
            process::exit(2);
        }
        let tv = tp.unwrap();
        if tv == 0 || tv > 600000 {
            print_usage();
            process::exit(2);
        }
        timeout_ms = tv;
    }
    let total_ms = timeout_ms.saturating_add(2000);
    let deadline = Instant::now() + Duration::from_millis(total_ms);
    let h2 = host.clone();
    let chan = std::sync::mpsc::channel();
    let tx = chan.0;
    let rx = chan.1;
    let _th = std::thread::spawn(move || {
        let r = run_probe(h2.as_str(), port, timeout_ms, deadline);
        let _ = tx.send(r);
    });
    let now2 = Instant::now();
    let wait_dur = if now2 >= deadline {
        Duration::from_millis(0)
    } else {
        deadline - now2
    };
    let got = rx.recv_timeout(wait_dur);
    if got.is_err() {
        print_failure("timed out");
        process::exit(0);
    }
    let res = got.unwrap();
    if res.is_ok() {
        let tup = res.unwrap();
        print_success(tup.0, tup.1, tup.2.as_str(), tup.4, tup.3.as_str());
        process::exit(0);
    } else {
        let e = res.err().unwrap();
        print_failure(e.as_str());
        process::exit(0);
    }
}

