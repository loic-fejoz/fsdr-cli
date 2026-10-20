#!/usr/bin/env python3
import sys
import socket
import struct
import time

def test_websocket(host="127.0.0.1", port=5006, timeout=5.0):
    print(f"[test_ws] Connexion TCP vers {host}:{port}...")
    try:
        sock = socket.create_connection((host, port), timeout=timeout)
    except Exception as e:
        print(f"[test_ws] ERREUR: Impossible d'établir la connexion TCP vers {host}:{port}: {e}")
        return False

    print("[test_ws] Envoi du handshake WebSocket HTTP 101...")
    handshake = (
        f"GET /ws HTTP/1.1\r\n"
        f"Host: {host}:{port}\r\n"
        f"Upgrade: websocket\r\n"
        f"Connection: Upgrade\r\n"
        f"Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n"
        f"Sec-WebSocket-Version: 13\r\n\r\n"
    )
    sock.sendall(handshake.encode("utf-8"))

    sock.settimeout(timeout)
    try:
        resp = sock.recv(4096).decode("utf-8", errors="ignore")
    except Exception as e:
        print(f"[test_ws] ERREUR: Aucune réponse HTTP reçue: {e}")
        sock.close()
        return False

    if "101 Switching Protocols" not in resp:
        print(f"[test_ws] ERREUR: Handshake rejeté par le serveur:\n{resp}")
        sock.close()
        return False

    print("[test_ws] Handshake 101 réussi ! En attente de trames FFT...")

    frames_received = 0
    start_time = time.time()
    
    while time.time() - start_time < timeout:
        try:
            header = sock.recv(2)
            if not header or len(header) < 2:
                time.sleep(0.05)
                continue
            
            b1, b2 = header[0], header[1]
            opcode = b1 & 0x0F
            payload_len = b2 & 0x7F
            
            if payload_len == 126:
                ext = sock.recv(2)
                payload_len = struct.unpack("!H", ext)[0]
            elif payload_len == 127:
                ext = sock.recv(8)
                payload_len = struct.unpack("!Q", ext)[0]
                
            payload = bytearray()
            while len(payload) < payload_len:
                chunk = sock.recv(payload_len - len(payload))
                if not chunk:
                    break
                payload.extend(chunk)

            # Opcode 2 = Binary frame
            if opcode == 2:
                floats_count = len(payload) // 4
                floats = struct.unpack(f"<{floats_count}f", payload[:floats_count*4])
                frames_received += 1
                min_val = min(floats) if floats else 0
                max_val = max(floats) if floats else 0
                print(f"[test_ws] Reçu trame #{frames_received} : {floats_count} bins FFT | Min: {min_val:.1f} dB, Max: {max_val:.1f} dB")
                if frames_received >= 3:
                    print(f"[test_ws] SUCCÈS : {frames_received} trames FFT reçues et validées avec succès !")
                    sock.close()
                    return True
        except socket.timeout:
            print("[test_ws] Timeout dépassé lors de l'attente d'une trame FFT.")
            break
        except Exception as e:
            print(f"[test_ws] Exception reçue: {e}")
            break

    sock.close()
    if frames_received == 0:
        print("[test_ws] ÉCHEC: Aucune trame FFT n'a été transmise par la WebSocket pendant le délai.")
        return False
    return True

if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 5006
    success = test_websocket(port=port)
    sys.exit(0 if success else 1)
