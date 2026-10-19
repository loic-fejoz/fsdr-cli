use anyhow::Result;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

pub type FreqChangeCallback = Arc<dyn Fn(u64) + Send + Sync + 'static>;

/// Hamlib rigctld CAT server for a channel
pub struct CatServer {
    port: u16,
    freq: Arc<AtomicU64>,
    ptt: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    on_freq_change: Option<FreqChangeCallback>,
}

impl CatServer {
    pub fn spawn(port: u16, initial_freq: u64) -> Result<Self> {
        Self::spawn_with_shared_freq(port, Arc::new(AtomicU64::new(initial_freq)), None)
    }

    pub fn spawn_with_shared_freq(
        port: u16,
        freq: Arc<AtomicU64>,
        on_freq_change: Option<FreqChangeCallback>,
    ) -> Result<Self> {
        let ptt = Arc::new(AtomicBool::new(false));
        let running = Arc::new(AtomicBool::new(true));

        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
        let bound_port = listener.local_addr()?.port();
        listener.set_nonblocking(true)?;

        let freq_c = freq.clone();
        let ptt_c = ptt.clone();
        let running_c = running.clone();
        let cb_outer = on_freq_change.clone();

        thread::spawn(move || {
            while running_c.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = stream.set_nodelay(true);
                        #[cfg(target_os = "linux")]
                        let _ = stream.set_nonblocking(true);
                        #[cfg(not(target_os = "linux"))]
                        let _ = stream.set_nonblocking(false);

                        let freq_inner = freq_c.clone();
                        let ptt_inner = ptt_c.clone();
                        let cb_inner = cb_outer.clone();
                        thread::spawn(move || {
                            let _ = handle_rigctld_client(stream, freq_inner, ptt_inner, cb_inner);
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(50));
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            port: bound_port,
            freq,
            ptt,
            running,
            on_freq_change,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn frequency(&self) -> u64 {
        self.freq.load(Ordering::Relaxed)
    }

    pub fn set_frequency(&self, new_freq: u64) {
        self.freq.store(new_freq, Ordering::Relaxed);
        if let Some(ref cb) = self.on_freq_change {
            cb(new_freq);
        }
    }

    pub fn freq_handle(&self) -> Arc<AtomicU64> {
        self.freq.clone()
    }

    pub fn is_ptt(&self) -> bool {
        self.ptt.load(Ordering::Relaxed)
    }
}

impl Drop for CatServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

const HAMLIB_DUMP_STATE: &str = r#"1
1
0
150000.000000 1500000000.000000 0x1ff -1 -1 0x17e00007 0x1f
0 0 0 0 0 0 0
150000.000000 1500000000.000000 0x1ff 5000 100000 0x17e00007 0x1f
0 0 0 0 0 0 0
0x1ff 1
0x1ff 0
0 0
0xc 2400
0xc 1800
0xc 3000
0xc 0
0x2 500
0x2 2400
0x2 50
0x2 0
0x10 300
0x10 2400
0x10 50
0x10 0
0x1 8000
0x1 2400
0x1 10000
0x20 15000
0x20 8000
0x40 230000
0 0
9990
9990
10000
0
10 
10 20 30 
0xffffffffffffffff
0xffffffffffffffff
0xfffffffff7ffffff
0xfffeff7083ffffff
0xffffffffffffffff
0xffffffffffffffbf
vfo_ops=0x7ffffff
ptt_type=0x1
targetable_vfo=0x10c3
has_set_vfo=1
has_get_vfo=1
has_set_freq=1
has_get_freq=1
has_set_conf=1
has_get_conf=1
has_power2mW=1
has_mW2power=1
has_get_ant=1
has_set_ant=1
timeout=0
rig_model=1
rigctld_version=Hamlib 4.7.2 2026-06-21T13:07:37Z SHA=40f63488f 64-bit
agc_levels=0=OFF 1=SUPERFAST 2=FAST 5=MEDIUM 3=SLOW 6=AUTO 4=USER
ctcss_list= 67.0 69.3 71.9 74.4 77.0 79.7 82.5 85.4 88.5 91.5 94.8 97.4 100.0 103.5 107.2 110.9 114.8 118.8 123.0 127.3 131.8 136.5 141.3 146.2 151.4 156.7 159.8 162.2 165.5 167.9 171.3 173.8 177.3 179.9 183.5 186.2 189.9 192.8 196.6 199.5 203.5 206.5 210.7 218.1 225.7 229.1 233.6 241.8 250.3 254.1
dcs_list= 17 23 25 26 31 32 36 43 47 50 51 53 54 65 71 72 73 74 114 115 116 122 125 131 132 134 143 145 152 155 156 162 165 172 174 205 212 223 225 226 243 244 245 246 251 252 255 261 263 265 266 271 274 306 311 315 325 331 332 343 346 351 356 364 365 371 411 412 413 423 431 432 445 446 452 454 455 462 464 465 466 503 506 516 523 526 532 546 565 606 612 624 627 631 632 654 662 664 703 712 723 731 732 734 743 754
level_gran=0=0,0,0;1=0,0,0;2=0,0,0;3=0,0,0;4=0,1,0.00392157;5=0,0,0;6=0,0,0;7=0,0,0;8=0,0,0;9=0,0,0;10=0,0,0;11=0,0,10;12=0.05,1,0.00195695;13=0,0,0;14=0,0,0;15=0,0,0;16=0,0,0;17=0,0,0;18=0,0,0;19=0,0,0;20=0,0,0;21=0,0,0;22=0,0,0;23=0,0,0;24=0,0,0;25=0,0,0;26=0,0,0;27=0,0,0;28=0,0,0;29=0,0,0;30=0,0,0;31=0,0,0;32=0,1,0.00392157;33=0,0,0;34=0,0,0;35=0,0,0;36=0,0,0;37=0,0,0;38=0,0,0;39=0,100,0.00392157;40=0,0,0;41=0,0,0;42=0,0,0;43=0,0,0;44=0,2,1;45=-30,10,0.5;46=0,3,1;47=0,0,0;48=0,0,0;49=0,0,0;50=0,0,0;51=0,0,0;52=0,0,0;53=0,0,0;54=0,0,0;55=0,0,0;56=0,0,0;57=0,0,0;58=0,0,0;59=0,0,0;60=0,0,0;61=0,0,0;62=0,0,0;63=0,0,0;
parm_gran=0=0,0,0;1=0,0,0;2=0,1,0.00392157;3=0,0,0;4=0,1065353216,998277249;5=0,0,0;6=0,0,0;7=0,0,0;8=0,0,0;9=0,0,0;10=BANDUNUSED,BAND70CM,BAND33CM,BAND23CM;11=STRAIGHT,BUG,PADDLE;12=1028443341,1065353216,989872160;13=0,0,0;14=0,0,0;15=0,0,0;16=0,0,0;17=0,0,0;18=0,0,0;19=0,0,0;20=0,0,0;21=0,0,0;22=0,0,0;23=0,0,0;24=0,0,0;25=0,0,0;26=0,0,0;27=0,0,0;28=0,0,0;29=0,0,0;30=0,0,0;31=0,0,0;32=0,1065353216,998277249;33=0,0,0;34=0,0,0;35=0,0,0;36=0,0,0;37=0,0,0;38=0,0,0;39=0,1120403456,998277249;40=0,0,0;41=0,0,0;42=0,0,0;43=0,0,0;44=0,2,1;45=-1041235968,1092616192,1056964608;46=0,3,1;47=0,0,0;48=0,0,0;49=0,0,0;50=0,0,0;51=0,0,0;52=0,0,0;53=0,0,0;54=0,0,0;55=0,0,0;56=0,0,0;57=0,0,0;58=0,0,0;59=0,0,0;60=0,0,0;61=0,0,0;62=0,0,0;63=0,0,0;
rig_model=1
hamlib_version=Hamlib 4.7.2 2026-06-21T13:07:37Z SHA=40f63488f 64-bit
done
"#;

fn handle_rigctld_client(
    mut stream: TcpStream,
    freq: Arc<AtomicU64>,
    ptt: Arc<AtomicBool>,
    on_freq_change: Option<FreqChangeCallback>,
) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();

    loop {
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = line.trim();
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.is_empty() {
                    line.clear();
                    continue;
                }

                let cmd = parts[0];
                let stripped_cmd = cmd.strip_prefix('\\').unwrap_or(cmd);

                match stripped_cmd {
                    "f" | "get_freq" => {
                        let current_freq = freq.load(Ordering::Relaxed);
                        writeln!(stream, "{current_freq}")?;
                    }
                    "F" | "set_freq" => {
                        if parts.len() > 1 {
                            if let Ok(new_freq) = parts[1].parse::<u64>() {
                                freq.store(new_freq, Ordering::Relaxed);
                                if let Some(ref cb) = on_freq_change {
                                    cb(new_freq);
                                }
                            }
                        }
                        writeln!(stream, "RPRT 0")?;
                    }
                    "t" | "get_ptt" => {
                        let current_ptt = if ptt.load(Ordering::Relaxed) { 1 } else { 0 };
                        writeln!(stream, "{current_ptt}")?;
                    }
                    "T" | "set_ptt" => {
                        if parts.len() > 1 {
                            let val = parts[1] == "1";
                            ptt.store(val, Ordering::Relaxed);
                        }
                        writeln!(stream, "RPRT 0")?;
                    }
                    "m" | "get_mode" => {
                        writeln!(stream, "USB\n2400")?;
                    }
                    "M" | "set_mode" => {
                        writeln!(stream, "RPRT 0")?;
                    }
                    "v" | "get_vfo" => {
                        writeln!(stream, "VFOA")?;
                    }
                    "V" | "set_vfo" => {
                        writeln!(stream, "RPRT 0")?;
                    }
                    "j" | "get_rit" => {
                        writeln!(stream, "0")?;
                    }
                    "J" | "set_rit" => {
                        if parts.len() > 1 {
                            if let Ok(offset) = parts[1].parse::<i64>() {
                                let current = freq.load(Ordering::Relaxed);
                                let new_freq = (current as i64 + offset).max(0) as u64;
                                freq.store(new_freq, Ordering::Relaxed);
                                if let Some(ref cb) = on_freq_change {
                                    cb(new_freq);
                                }
                            }
                        }
                        writeln!(stream, "RPRT 0")?;
                    }
                    "s" | "get_split_vfo" => {
                        writeln!(stream, "0\nVFOA")?;
                    }
                    "S" | "set_split_vfo" => {
                        writeln!(stream, "RPRT 0")?;
                    }
                    "chk_vfo" => {
                        writeln!(stream, "0")?;
                    }
                    "dump_state" => {
                        write!(stream, "{HAMLIB_DUMP_STATE}")?;
                    }
                    "get_powerstat" => {
                        writeln!(stream, "1")?;
                    }
                    "set_powerstat" => {
                        writeln!(stream, "RPRT 0")?;
                    }
                    "l" | "get_level" => {
                        writeln!(stream, "0")?;
                    }
                    "L" | "set_level" => {
                        writeln!(stream, "RPRT 0")?;
                    }
                    "u" | "get_func" => {
                        writeln!(stream, "0")?;
                    }
                    "U" | "set_func" => {
                        writeln!(stream, "RPRT 0")?;
                    }
                    "p" | "get_parm" => {
                        writeln!(stream, "0")?;
                    }
                    "P" | "set_parm" => {
                        writeln!(stream, "RPRT 0")?;
                    }
                    "q" => {
                        writeln!(stream, "RPRT 0")?;
                        break;
                    }
                    _ => {
                        writeln!(stream, "RPRT 0")?;
                    }
                }
                stream.flush()?;
                line.clear();
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(std::time::Duration::from_millis(1));
            }
            Err(_) => break,
        }
    }
    Ok(())
}

/// Proxy client translating rigctld commands from external rig (e.g. FT-817)
/// with LO offset and multiplier: f_target = (f_rf - lo_offset) / lo_multiplier
pub struct CatClientProxy {
    running: Arc<AtomicBool>,
}

impl CatClientProxy {
    pub fn spawn(
        target_addr: String,
        lo_offset: i64,
        lo_multiplier: f64,
        freq_source: Arc<AtomicU64>,
    ) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_c = running.clone();

        thread::spawn(move || {
            while running_c.load(Ordering::Relaxed) {
                if let Ok(mut stream) = TcpStream::connect(&target_addr) {
                    let mut last_reported_freq = 0u64;
                    while running_c.load(Ordering::Relaxed) {
                        let cur_freq = freq_source.load(Ordering::Relaxed);
                        if cur_freq != last_reported_freq && cur_freq > 0 {
                            // Translate RF frequency to IF frequency for rig
                            let if_freq = ((cur_freq as f64 - lo_offset as f64) / lo_multiplier)
                                .round() as i64;
                            if if_freq > 0 {
                                let cmd = format!("F {}\n", if_freq);
                                if stream.write_all(cmd.as_bytes()).is_err() {
                                    break;
                                }
                                last_reported_freq = cur_freq;
                            }
                        }
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
                thread::sleep(std::time::Duration::from_secs(2));
            }
        });

        Self { running }
    }
}

impl Drop for CatClientProxy {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
