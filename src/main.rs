#[derive(Debug, Clone, Copy)]
enum PktType {
    UDP,
    CNP,
    END,
}

struct Packet {
    pub pkt_type: PktType,
    pub recv_time: isize,
}

impl Packet {
    fn new(pkt_type: PktType, recv_time: isize) -> Self {
        Self {
            pkt_type,
            recv_time,
        }
    }
}

struct Dcqcn {
    pub g: f64,
    pub alpha: f64,
    pub alpha_timer: usize,
    pub alpha_time_counter: usize,
    pub current_rate: f64,
    pub target_rate: f64,
    pub increase_timer: usize,
    pub increase_time_counter: usize,
    pub t: usize,
    pub f: usize,
    pub ai_rate: f64,
    pub hai_rate: f64,
    pub bandwidth: f64,
}

impl Dcqcn {
    fn new(
        g: f64,
        alpha_timer: usize,
        increase_timer: usize,
        f: usize,
        ai_rate: f64,
        hai_rate: f64,
        bandwidth: f64,
    ) -> Self {
        Self {
            g,
            alpha: 1.0,
            alpha_timer,
            alpha_time_counter: 0,
            current_rate: bandwidth,
            target_rate: 0.0,
            increase_timer,
            increase_time_counter: 0,
            t: 0,
            f,
            ai_rate,
            hai_rate,
            bandwidth,
        }
    }

    fn cut_rate(&mut self) {
        self.target_rate = self.current_rate;
        self.current_rate = self.current_rate * (1.0 - self.alpha / 2.0);
        self.alpha = (1.0 - self.g) * self.alpha + self.g;

        self.alpha_time_counter = 0;
        self.increase_time_counter = 0;
    }

    fn update_alpha(&mut self) {
        self.alpha = (1.0 - self.g) * self.alpha;
    }

    fn fast_recovery(&mut self) {
        self.current_rate = (self.current_rate + self.target_rate) / 2.0;

        if self.current_rate > self.bandwidth {
            self.current_rate = self.bandwidth;
        }
    }

    fn additive_increase(&mut self) {
        self.target_rate += self.ai_rate;
        self.current_rate = (self.current_rate + self.target_rate) / 2.0;

        if self.current_rate > self.bandwidth {
            self.current_rate = self.bandwidth;
        }
    }

    fn hyper_increase(&mut self) {
        self.target_rate += self.hai_rate;
        self.current_rate = (self.current_rate + self.target_rate) / 2.0;

        if self.current_rate > self.bandwidth {
            self.current_rate = self.bandwidth;
        }
    }
}

fn generate_pakcets() -> Vec<Packet> {
    vec![Packet::new(PktType::CNP, 1), Packet::new(PktType::END, 100)]
}

fn main() {
    let mut timer = 0;

    let g = 1.0 / 256.0;
    let bandwidth = 102400.0;
    let alpha_timer = 55;
    let increase_timer = 55;
    let ai_rate = 40.0;
    let hai_rate = 100.0;
    let f = 5;

    let mut dcqcn = Dcqcn::new(
        g,
        alpha_timer,
        increase_timer,
        f,
        ai_rate,
        hai_rate,
        bandwidth,
    );

    let recv_pkts = generate_pakcets();

    let mut pkt_iter = recv_pkts.iter();

    if recv_pkts.len() == 0 {
        panic!("Don't have any packets.");
    }

    let mut packet = pkt_iter.next().unwrap();

    loop {
        println!("time: {}, Rc is: {}", timer, dcqcn.current_rate);

        let next_time = packet.recv_time;

        if next_time == timer {
            match packet.pkt_type {
                PktType::CNP => {
                    dcqcn.cut_rate();
                }
                PktType::UDP => {
                    todo!();
                }
                PktType::END => {
                    break;
                }
            }

            match pkt_iter.next() {
                Some(next_pkt) => {
                    packet = next_pkt;
                }
                None => {
                    break;
                }
            }
        }

        timer += 1;
        dcqcn.alpha_time_counter += 1;
        dcqcn.increase_time_counter += 1;

        if dcqcn.alpha_time_counter > dcqcn.alpha_timer {
            dcqcn.update_alpha();
            dcqcn.alpha_time_counter = 0;
        }

        if dcqcn.increase_time_counter > dcqcn.increase_timer {
            dcqcn.t += 1;
            dcqcn.increase_time_counter = 0;

            if dcqcn.t < dcqcn.f {
                dcqcn.fast_recovery();
            } else if dcqcn.t == dcqcn.f {
                dcqcn.additive_increase();
            } else {
                dcqcn.hyper_increase();
            }
        }
    }
}
