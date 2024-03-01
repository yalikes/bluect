<script lang="ts">
    import { Device } from "./device";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api";
    export let this_device: Device;
    onMount(() => {});
    function disconnect() {
        invoke("disconnect_device", { mac_addr: this_device.mac_addr }).then(
            (result) => {},
        );
    }
    function connect_device() {
        invoke("connect_device", { mac_addr: this_device.mac_addr }).then(
            (result) => {},
        );
    }
</script>

{#if this_device.is_connected}
    <div
        class="p-2 my-2 bg-cyan-200 rounded-md hover:rounded-lg hover:transition-all"
    >
        <div class="flex justify-center">
            <div class="w-[12rem]"></div>
            <div class=" inline-block">
                <p class=" text-2xl">
                    {this_device.name}
                </p>
                <p>
                    {this_device.mac_addr}
                </p>
            </div>
            <div class="flex w-[12rem] justify-center">
                <button
                    on:click={disconnect}
                    class=" bg-gray-500 hover:bg-gray-400 active:bg-gray-200 active:text-black rounded-md p-2 text-white"
                    >disconnect</button
                >
            </div>
        </div>
    </div>
{:else}
    <button class=" my-2 block shadow-lg w-full active:shadow-sm" on:click={connect_device}>
        <div
            class="p-2 bg-gray-200 hover:bg-gray-300 rounded-md hover:rounded-lg hover:transition-all"
        >
            <p class=" text-2xl">
                {this_device.name}
            </p>
            <p>
                {this_device.mac_addr}
            </p>
        </div>
    </button>
{/if}

<style lang="postcss">
</style>
