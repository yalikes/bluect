<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  class Device{
    mac_addr: string;
    name: string;
    constructor(mac_addr: string, name: string){
      this.mac_addr = mac_addr;
      this.name = name;
    }
  }
  let devices: Device[] = [];
  function get_devices() {
    invoke("get_devices", {}).then((result) => {
      (result as Device[]).forEach((o) => {
        console.log(o);
        devices.push(new Device(o.mac_addr, o.name));
      });
      devices = devices;
    });
  }
  function refresh_devices() {
    invoke("refresh_devices", {}).then((result) => {
      console.log("refresh_devices: ", result);
    });
  }
</script>

<main class="container">
  <h1>devices</h1>
  <div>
    {#each devices as d}
      { d.name }
      { d.mac_addr }
    {/each}
  </div>
  <button on:click={refresh_devices}> refresh </button>
  <button on:click={get_devices}> get devices </button>
</main>

<style>
</style>
