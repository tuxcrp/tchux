import socket

server_address = ('localhost', 12345)


def main():

    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        sock.connect(server_address)
        name = input("Name: ")
        sock.sendall(name.encode())

        response = sock.recv(1024).decode('utf-8')
        print(f"Received from server: {response}")

    except Exception as e:
        print(f"Error: {e}")

    while True:
        message = input("Message: ")
        sock.sendall(message.encode('utf-8'))


main()
