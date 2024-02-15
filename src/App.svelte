<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri'
  let devices: any[] = [];
  function get_devices(){
    console.log("in refresh!");
    invoke("get_devices", {}).then((result) => {
      console.log("get_devices: ",result);
    });
  }
  function refresh_devices(){
    console.log("in refresh!");
    invoke("refresh_devices", {}).then((result) => {
      console.log("refresh_devices: ", result);
    });
  }
</script>
<main class="container">
  <h1>devices</h1>
  <div>
    {#each devices as d}
      d
    {/each}
  </div>
  <button on:click={refresh_devices}>
    refresh
  </button>
  <button on:click={get_devices}>
    get devices
  </button>
</main>

<style>
</style>