import utime
import re
import asyncio

class SimException(Exception):
    def __init__(self, desc=""):
        self.desc = desc

def truncate_response(r):
    if len(r) < 50:
        return r
    return r[:47]+"..."

class Sim:
    def __init__(self, uart):
        self.uart = uart
        self.http_checkpoint = 0
    
    async def send_command(self, command, timeout_ms=1000, accept_response=None, response_max_len=100):
        await asyncio.sleep_ms(100)
        await self.flush_receive_buffer()
        
        print("SIM7600 << "+repr(command))
        
        self.uart.write(command + '\r')
        if timeout_ms is None:
            return None
        
        # For some reason responses desync without this delay
        await asyncio.sleep_ms(100)
        
        return await self.read_response(timeout_ms=timeout_ms, accept_response=accept_response,
                response_max_len=response_max_len)
    
    async def read_response(self, timeout_ms=1000, accept_response=None, response_max_len=100):
        print("SIM7600 >> ", end='')
        start_time = utime.ticks_ms()
        response = b''
        response_accepted_time = None
        while (utime.ticks_diff(utime.ticks_ms(), start_time) < timeout_ms):
            if self.uart.any():
                r = self.uart.read(self.uart.any())
                print(r, end=' ')
                response += r
                
                if response_accepted_time is None:
                    if type(accept_response) == str:
                        if accept_response in response:
                            response_accepted_time = utime.ticks_ms()
                    elif type(accept_response) == list:
                        for r in accept_response:
                            if r in response:
                                response_accepted_time = utime.ticks_ms()
                                break
                        
                if len(response) >= response_max_len:
                    raise SimException(truncate_response(response))
            
            # This is a big of a kludge:
            # Because accept_response doesn't actually contain the
            # final characters of the response (e.g. variable
            # parameters), we have to wait for some time after receiving
            # it, so that we are likely to have actually received
            # everything
            if response_accepted_time is not None:
                if utime.ticks_diff(utime.ticks_ms(), response_accepted_time) >= 100:
                    break
                    
            await asyncio.sleep_ms(10)
        print('') # Newline
        return response
    
    async def flush_receive_buffer(self):
        printed = False
        while self.uart.any():
            if not printed:
                print("SIM7600: Flushing receive buffer: ", end='')
                printed = True
            r = self.uart.read(self.uart.any())
            print(r, end=' ')
        if printed:
            print('') # Newline
        
    async def http_get(self, url, timeout_ms=30000, response_max_len=1000):
        # SIM7500_SIM7600_Series_HTTP(S)_Application_Note_V2.00.pdf
        print("SIM7600: http_get")
        start_time = utime.ticks_ms()
        self.http_checkpoint = 0
        
        await self.flush_receive_buffer()
        
        # Wait for a connection for 30 timeouts
        for i in range(0, 60):
            response = await self.send_command("AT+CPIN?", accept_response="OK\r\n") # SIM card status
            if response == b'AT+CPIN?\r\r\n+CPIN: READY\r\n\r\nOK\r\n':
                break
        else:
            raise SimException("Invalid CPIN response: "+repr(truncate_response(response)))
        
        self.http_checkpoint = 10
            
        response = await self.send_command("AT+CSQ", accept_response="OK\r\n") # RF signal
        # E.g. b'AT+CSQ\r\r\n+CSQ: 24,99\r\n\r\nOK\r\n'
        # TODO: Parse response
        
        response = await self.send_command("AT+CGREG?", accept_response="OK\r\n") # PS service
        # Accept home or roaming
        if response not in [b'AT+CGREG?\r\r\n+CGREG: 0,1\r\n\r\nOK\r\n', b'AT+CGREG?\r\r\n+CGREG: 0,5\r\n\r\nOK\r\n']:
            raise SimException("Invalid CGREG response: "+repr(truncate_response(response)))
        
        self.http_checkpoint = 20

        # Set TCP timeouts <netopen_timeout>,<cipopen_timeout>,<cipsend_timeout> (default are 120s each)
        # TODO: Figure out whether these apply to AT+HTTPACTION or not
        response = await self.send_command("AT+CIPTIMEOUT=30000,30000,30000", accept_response=["OK\r\n"])

        response = await self.send_command("AT+COPS?", accept_response="OK\r\n") # Network information
        # E.g. b'AT+COPS?\r\r\n+COPS: 0,0,"elisa elisa",7\r\n\r\nOK\r\n'
        # TODO: Parse response
        
        # NOTE: We get ERROR here in prefectly functional cases so both responses are fine
        response = await self.send_command("AT+CGACT=0,1", accept_response=["OK\r\n", "ERROR\r\n"]) # Activate network bearing
        # E.g. b'AT+CGACT=0,1\r\r\nERROR\r\n'
        # TODO: Parse response

        response = await self.send_command("AT+CGACT?", accept_response="OK\r\n")
        # No idea what this response means
        # We'll just ignore it and let the process fail at AT+HTTPINIT in case there's a problem
        #if response != b'AT+CGACT?\r\r\n+CGACT: 1,1\r\n+CGACT: 2,0\r\n+CGACT: 3,0\r\n\r\nOK\r\n':
        #    raise SimException(truncate_response(response))
        
        # Call AT+HTTPTERM in case a previous connection wasn't cleaned up properly
        self.http_checkpoint = 28
        await self.send_command("AT+HTTPTERM", accept_response="OK\r\n")
        self.http_checkpoint = 29
        
        self.http_checkpoint = 30

        response = await self.send_command("AT+HTTPINIT", accept_response=["OK\r\n", "ERROR\r\n"], timeout_ms=13000)
        if response != b'AT+HTTPINIT\r\r\nOK\r\n':
            raise SimException(truncate_response(response))

        self.http_checkpoint = 40

        response = await self.send_command('AT+HTTPPARA="URL","'+url+'"', accept_response="OK\r\n", response_max_len=100+len(url))
        # TODO: Parse response

        # Send a GET request (=1 would be POST)
        response = await self.send_command("AT+HTTPACTION=0", timeout_ms=timeout_ms, accept_response="HTTPACTION:")
        match = re.search(br'HTTPACTION:\s*\d+,\s*(\d+),\s*(\d+)', response)
        if not match:
            raise SimException("Invalid HTTPACTION response: "+repr(truncate_response(response)))
        status_code = match.group(1)
        content_length = match.group(2)
        print("status_code: {}, content_length: {}".format(status_code, content_length))

        self.http_checkpoint = 50

        # Read the status code and content length from the header here, because this response is less likely to be filled with errors caused by RF activity
        #response = await self.send_command("AT+HTTPHEAD")

        response = await self.send_command("AT+HTTPREAD=0,"+str(response_max_len), response_max_len=response_max_len+100) # Read bytes
        header = b'AT+HTTPREAD=0,'+str(response_max_len)+'\r\r\nOK\r\n\r\n+HTTPREAD: DATA,'+str(response_max_len)+'\r\n'
        # There's often junk at the beginning of the response to this, so find the header after the junk and hope there's no junk after the header
        n = response.find(header)        
        if n == -1:
            raise SimException("Can't find header: "+repr(truncate_response(response)))
        self.http_checkpoint = 60

        response = response[n:]
        end_marker = b'\r\n+HTTPREAD: 0\r\n'
        data_end = response.rfind(end_marker)
        if data_end == -1:
            raise SimException("Can't find end marker: "+repr(truncate_response(response)))
        content = response[len(header):data_end]

        self.http_checkpoint = 70

        response = await self.send_command("AT+HTTPTERM", accept_response="OK\r\n")

        self.http_checkpoint = 80

        return status_code, content_length, content

