import gc
from machine import Pin, Timer, UART
import select
import sys
from time import ticks_diff, ticks_ms

from Cluster import Cluster
from ModuleGpio import ModuleGpio
from UartProtocol import UartProtocol

picow = True
try:
    import network
except ImportError:
    picow = False

cluster = Cluster(Pin(3, Pin.OUT, value=0), [
    ModuleGpio(2, 28, 27, 26, 22, 1),
    ModuleGpio(14, 18, 19, 20, 21, 0),
    ModuleGpio(1, 9, 8, 7, 6, 0),
    ModuleGpio(15, 10, 11, 12, 13, 1)
])

TIMEOUT_MS = const(200)

uart_input = UartProtocol(
    UART(0,
         baudrate=38400,
         tx=Pin(16, Pin.IN, Pin.PULL_UP),
         rx=Pin(17, Pin.OUT)))
uart_output = UartProtocol(
    UART(1, baudrate=38400, tx=Pin(4, Pin.IN, Pin.PULL_UP), rx=Pin(5,
                                                                   Pin.OUT)))

led = machine.Pin("LED", machine.Pin.OUT)


def blink(timer):
    led.toggle()


timer = Timer()
timer.init(freq=2.5, mode=Timer.PERIODIC, callback=blink)

test_chars = list(" ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:.-?!$&#") + list(
    reversed("ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:.-?!$&# "))

# todo: duplicate letters until I have printed more modules
test_words = list(
    map(lambda x: ''.join([char * 4 for char in x]), [
        "in", "of", "to", "is", "it", "on", "no", "us", "at", "un", "go", "an",
        "my", "up", "me", "as", "he", "we", "so", "be", "by", "or", "do", "if",
        "hi", "bi", "ex", "ok", "18", "21", "99", "$$", "&&", "##"
    ]))

#message = list(map(lambda s: s * num_modules, test_chars))
queue = 100 * test_words if picow else []

poll = select.poll()
poll.register(uart_input.uart, select.POLLIN)

seq_in = 0
seq_out = 0

while True:
    try:
        gc.collect()

        max_steps_in = 0  # Todo add to queue

        for sock, event in poll.ipoll(1000):
            if event and select.POLLIN:
                if sock == uart_input.uart:
                    frame = uart_input.uart_read(UartProtocol.CMD_SET)
                    if frame:
                        seq_in = frame.seq
                        queue = [frame.letters.decode('ascii')]
                        max_steps_in = frame.steps

        if not queue:
            continue

        gc.collect()
        print('mem_free:', gc.mem_free())

        seq_out += 1

        letters = queue.pop(0)
        letters_overflow = letters[cluster.num_modules():]
        print('letters:', letters, letters_overflow)

        cluster.set_letters(letters)
        max_steps = max([cluster.get_max_steps(), max_steps_in])

        if letters_overflow:
            uart_output.uart_write(UartProtocol.CMD_SET, seq_out, max_steps,
                                   letters_overflow)

            frame = uart_output.uart_read(UartProtocol.CMD_ACK, seq_out,
                                          TIMEOUT_MS)
            if frame:
                max_steps = frame.steps

        uart_input.uart_write(UartProtocol.CMD_ACK, seq_in, max_steps, '')

        cluster.rotate_until_stopped(max_steps)

        status = cluster.get_status()

        if letters_overflow:
            frame = uart_output.uart_read(UartProtocol.CMD_END, seq_out,
                                          TIMEOUT_MS)
            status = '{},{}'.format(status, frame.letters if frame else '?')

        print('status:', status)
        uart_input.uart_write(UartProtocol.CMD_END, seq_in, 0, status)
    except Exception as e:
        sys.print_exception(e)
