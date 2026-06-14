use std::io::{self, Write, Stdout};
use std::thread;
use std::time::Duration;

const CLR: &str = "\x1b[2J\x1b[1;1H";
const HIDE: &str = "\x1b[?25l";
const SHOW: &str = "\x1b[?25h";
const RST: &str = "\x1b[0m";
const GRN: &str = "\x1b[32m";
const BLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const UND: &str = "\x1b[4m";

const PURPLE: &str = "\x1b[48;2;147;0;211m";
const ORANGE: &str = "\x1b[48;2;255;140;0m";
const GRAY: &str = "\x1b[48;2;80;80;80m";
const BLACK: &str = "\x1b[30m";

fn grad(p: f32) -> String {
    let r = (128.0 + 127.0 * p) as u8;
    let g = (0.0 + 140.0 * p) as u8;
    let b = (128.0 - 128.0 * p) as u8;
    format!("\x1b[38;2;{r};{g};{b}m")
}

fn bar_grad(i: usize, total: usize) -> String {
    let p = i as f32 / total as f32;
    let r = (147.0 + 108.0 * p) as u8;
    let g = (0.0 + 154.0 * p) as u8;
    let b = (211.0 - 211.0 * p) as u8;
    format!("\x1b[38;2;{r};{g};{b}m")
}

fn bar(filled: usize, total: usize, w: usize) -> String {
    let n = (filled * w) / total;
    let mut s = String::with_capacity(w * 20);
    for i in 0..w {
        if i < n {
            let c = bar_grad(i, w);
            s.push_str(&format!("{c}█{RST}"));
        } else {
            s.push('░');
        }
    }
    s
}

fn flush(o: &mut Stdout) {
    let _ = o.flush();
}

fn badge(text: &str, bg: &str) -> String {
    format!("{bg}{BLACK}{UND} {text} {RST}")
}

fn animate(o: &mut Stdout) -> io::Result<()> {
    print!("{HIDE}");
    flush(o);

    let faces = [
        "(◕‿◕)", "(◠‿◠)", "(◕‿◕)", "(⊙‿⊙)", 
        "(◕‿◕)", "(¬‿¬)", "(◕‿◕)", "(✿‿✿)",
        "(◕‿◕)", "(◠‿◠)", "(◕‿◕)", "(☆‿☆)",
    ];
    
    let actions = [
        "Launch sequence initiated.",
        "Calibrating tentacles...",
        "Filling ink tanks...",
        "Syncing with nebula...",
        "Warming up reactor...",
        "Almost there...",
    ];

    for prog in (0..=100).step_by(4) {
        let frame = (prog / 4) % faces.len();
        let action = actions[(prog / 4) % actions.len()];
        let b = bar(prog, 100, 25);
        let c = grad(prog as f32 / 100.0);
        
        print!("\r  {c}{}{RST}  [{b}]  {c}{:>3}%{RST}  {BLD}{action}{RST}", faces[frame], prog);
        flush(o);

        if prog < 100 {
            thread::sleep(Duration::from_millis(180));
        }
    }
    
    print!("\r\x1b[K");
    flush(o);
    
    Ok(())
}

fn ask_dir(o: &mut Stdout) -> io::Result<String> {
    let c = grad(1.0);
    println!("{GRN}✔{RST} iniciando argo! {c}(ﾉ◕ヮ◕)ﾉ*:･ﾟ✧{RST}\n");
    println!(" {} Where should we create your new project?", badge("dir", PURPLE));
    print!("  {DIM}./{RST}");
    flush(o);
    print!("{SHOW}");
    flush(o);

    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s.trim().to_string())
}

fn ask_git(o: &mut Stdout) -> io::Result<bool> {
    println!("\n {} Initialize a new git repository?", badge("git", ORANGE));
    print!("  {DIM}[y/n]{RST} ");
    flush(o);

    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    let ans = s.trim().to_lowercase();
    Ok(ans == "y" || ans == "yes")
}

fn show_result(name: &str, git: bool) {
    let n = grad(1.0);
    
    println!("\n{} {BLD}Project initialized!{RST}\n", badge("argo", GRAY));
    println!("  {GRN}✔{RST} Template copied");
    if git {
        println!("  {GRN}✔{RST} Git initialized");
    }
    
    // Carita + Fair winds (despedida náutica del Argo)
    println!("\n  {n}(◕‿◕){RST}  {DIM}Fair winds:{RST}  {BLD}May your code sail smooth!{RST}");
    
    // Next steps para lenguaje de programación (tipo Lua)
    println!("\n{} {BLD}Liftoff confirmed. Start coding!{RST}\n", badge("next", GRAY));
    println!("  Enter your project directory using {DIM}cd ./{name}{RST}");
    println!("  Run {DIM}argo run{RST} to execute your program.");
    println!("  Run {DIM}argo test{RST} to run your test suite.");
    println!("  Run {DIM}argo build{RST} to compile for production.");
    println!("  Run {DIM}argo repl{RST} for an interactive session.");
}

fn main() -> io::Result<()> {
    let mut o = io::stdout();
    print!("{CLR}");
    flush(&mut o);
    
    println!("  {BLD}Argo{RST} {DIM}Launch sequence initiated.{RST}\n");
    
    animate(&mut o)?;
    let name = ask_dir(&mut o)?;
    let git = ask_git(&mut o)?;
    
    show_result(&name, git);
    
    print!("{SHOW}");
    flush(&mut o);
    
    Ok(())
}