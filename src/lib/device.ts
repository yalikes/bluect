class Device {
  mac_addr: string;
  name: string;
  is_connected: boolean = false;
  constructor(mac_addr: string, name: string, is_connected?: boolean) {
    this.mac_addr = mac_addr;
    this.name = name;
    if(!(is_connected === undefined)){
      this.is_connected = is_connected;
    }
  }
}

export { Device }