// Converts an image to braille art

fn show_usage(bad: bool) -> ! {
	println!(
"Usage: brimg <input> [options] [ <output> ]

Options:
  -h, --help                    Print help
  -i, --invert                  Invert output
  -q, --quiet                   Do not print output
  -f, --filter                  Use filtering
  -s, --size <width> <height>   Output size (in characters)
  -t, --thresh <threshold>      Set threshold (ignored if dithering enabled)
  -d, --dither <min> <max>      Enable dithering mode"
);
	std::process::exit(bad as i32)
}

struct Config {
	input: String,
	help: bool,
	invert: bool,
	quiet: bool,
	filter: bool,
	size: (usize, usize),
	thresh: u8,
	dither: Option<(u8, u8)>,
	output: Option<String>,
}

impl Default for Config {
	fn default() -> Self { Self {
		input: String::new(),
		help: false,
		invert: false,
		quiet: false,
		filter: false,
		size: (64, 27),
		thresh: 128,
		dither: None,
		output: None,
	}}
}

impl Config {
	fn load_from_args() -> Option<Self> {
		let mut args = std::env::args().skip(1);
		let mut cfg = Self {
			input: args.next()?,
			..Default::default()
		};
		
		while let Some(arg) = args.next() { if arg.starts_with("-") { match arg.as_str() {
			"-h" | "--help" => cfg.help = true,
			"-i" | "--invert" => cfg.invert = true,
			"-q" | "--quiet" => cfg.quiet = true,
			"-f" | "--filter" => cfg.filter = true,
			"-s" | "--size" => cfg.size = (
				args.next()?.parse().ok()?,
				args.next()?.parse().ok()?,
			),
			"-t" | "--thresh" => cfg.thresh = args.next()?.parse().ok()?,
			"-d" | "--dither" => cfg.dither = Some((
				args.next()?.parse().ok()?,
				args.next()?.parse().ok()?
			)),
			_ => return None,
		}} else {
			if args.next().is_some() { return None }
			cfg.output = Some(arg);
			break;
		}}
		
		Some(cfg)
	}
}

fn main() {
	let Some(cfg) = Config::load_from_args() else { show_usage(true) };
	if cfg.help { show_usage(false) }
	
	let img = match image::open(&cfg.input) {
		Ok(img) => img.into_luma8(),
		Err(e) => {
			eprintln!("Failed to open {:?}:\n{e}", cfg.input);
			std::process::exit(1);
		}
	};
	
	let dw = cfg.size.0 as u32 * 2;
	let dh = cfg.size.1 as u32 * 4;
	
	use image::imageops as iop;
	let img = iop::resize(&img, dw, dh, match cfg.filter {
		false => iop::FilterType::Nearest,
		true => iop::FilterType::CatmullRom,
	});
	
	let mut out = vec![0u32; cfg.size.0 * cfg.size.1];
	for (i, c) in out.iter_mut().enumerate() {
		let cx = i % cfg.size.0;
		let cy = i / cfg.size.0;
		const DX: [u32; 8] = [0, 0, 0, 1, 1, 1, 0, 1];
		const DY: [u32; 8] = [0, 1, 2, 0, 1, 2, 3, 3];
		
		for i in 0..8 {
			let dx = cx as u32 * 2 + DX[i];
			let dy = cy as u32 * 4 + DY[i];
			let p = img.get_pixel(dx, dy).0[0];
			
			if let Some((lo, hi)) = cfg.dither {
				if p >= rand::random_range(lo..hi) {
					*c |= 1 << i;
				}
			} else if p >= cfg.thresh {
				*c |= 1 << i;
			}
		}
		
		if cfg.invert {
			*c ^= 0xFF;
		}
	}
	
	let out = out.into_iter()
		.map(|b| char::from_u32(0x2800 | b).unwrap())
		.collect::<Vec<_>>()
		.chunks(cfg.size.0)
		.map(|c| c.into_iter().collect::<String>())
		.collect::<Vec<_>>()
		.join("\n");
	
	if !cfg.quiet { println!("{out}") }
	if let Some(output) = cfg.output {
		match std::fs::write(&output, out + "\n") {
			Ok(()) => (),
			Err(e) => {
				eprintln!("Failed to write {output:?}:\n{e}");
				std::process::exit(1);
			}
		}
	}
}
