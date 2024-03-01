<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { Device } from "./lib/device";
  import DeviceElement from "./lib/DeviceElement.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  let devices: Device[] = [];
  function get_devices() {
    invoke("get_devices", {}).then((result) => {
      devices = [];
      (result as Device[]).forEach((o) => {
        devices.push(new Device(o.mac_addr, o.name, o.is_connected));
      });
      console.log("get_devices", devices);
      devices = devices;
    });
  }
  function refresh_devices() {
    invoke("refresh_devices", {}).then((result) => {
      console.log("refresh_devices");
    });
  }
  onMount(() => {
    const unlisten = listen("update_devices", () => {
      get_devices();
    });
  });
</script>

<main class="container">
  <h1 class=" text-3xl">devices</h1>
  <button on:click={refresh_devices} class="btn"> refresh </button>
  <button on:click={get_devices} class="btn"> get devices </button>
  <div>
    {#each devices as d}
      <DeviceElement this_device={d} />
    {/each}
  </div>
</main>

<style lang="postcss">
  .btn {
    @apply text-xl border rounded-md p-2 shadow m-2 hover:font-bold hover:shadow-lg active:translate-y-2 active:shadow-sm;
  }
</style>
