<script lang="ts">

	import { invoke } from "@tauri-apps/api/tauri";
	import { onMount,  } from "svelte";		
	import { slide } from 'svelte/transition';
	import * as eases from 'svelte/easing';
	import { iv2, ready } from "./lib/com";

	let selected = -1;
	let selected_q = "";

	onMount(async () => {
		await ready;
		let out_come = await iv2("search",{"q":"lv999"});
		let out_come2 = await iv2("search_id",{"q":14111});
		console.log("out_come",out_come)
		console.log("out_come2",out_come2)
	});


	let results = null;
	let selected_show_data = null;

	function search(eq){
		console.log("search",eq)
		results = iv2("search",{q:eq}).then(r=>{
			console.log("r",r)
			return r
		})
	}
	async function lookup_sel(x) {
		await iv2("set_selected",{id:x,query:selected_q}).catch(e=>{console.log(e)})
		await iv2("lookup_selected",{}).then(x=>{
			console.log("lookup",x)
			selected_show_data = x;
		},e=>{
			console.log(e)
		})
	}
	let search_query = "kaguya-sama";
	$: {
		search(search_query)
		if (selected != -1) {
			lookup_sel(selected)
		}
	}
	let hide_search = false;
</script>

<main class="p-6 text-neutral-300">
  <h1 class="text-white text-3xl pb-2 relative" >Embayro</h1>
  {#await ready}
	
	initalizing


  	{:then d} 
		<div class="w-64">
			<input type="text" class="w-full border-neutral-600 border rounded-md appearance-none bg-neutral-900 px-4 py-1 shadow-sm hover:ring-1 focus:outline-none focus:ring-1 ring-pink-300 focus:ring-pink-300" 
				placeholder="Search for an anime.." bind:value={search_query} on:focus={e=>{selected = -1; hide_search=false}} on:blur={e=>{setTimeout(()=>{hide_search=true},200);}}
			/>
			{#if results && !hide_search && selected == -1}
				<div transition:slide="{{delay:0, easing:eases.cubicInOut}}" class="bg-neutral-900 mt-2 rounded-xl w-96 shadow-2xl p-2 h-80 overflow-y-scroll scrollable absolute" >

					{#await results then r}

						{#each r as ani}
						<div class="py-3 w-full hover:cursor-pointer transition-colors hover:text-pink-300" on:click={e=>{selected = ani.entry.id; selected_q= ani.entry.title}} >
							<p>{ani.entry.title}</p>
						</div>
						<hr class="border-neutral-800 last:hidden" />
						{/each}
					{/await}
				</div>			
			{/if}
			
		</div>
		{#if selected_show_data}
			<div class="w-full flex flex-row mt-5">
				<div class="flex flex-[.2] flex-wrap" >
					<img width="64" src="{selected_show_data.image.original}" alt="{selected_show_data.name}" class="rounded-xl w-full aspect-auto object-cover max-w-sm" >

				</div>
				<div class="flex flex-[.8] flex-col ml-3" >
					<h1 class="text-2xl" >{selected_show_data.name}</h1>
					<h3 class="text-xl">About</h3>
					{@html selected_show_data.summary}
					<h3 class="text-xl">Status</h3>
					<p>• {selected_show_data.status}</p>
					<h3 class="text-xl">Genres</h3>
					{#each selected_show_data.genres as genre}
						<p> • {genre}</p>
					{/each}
				</div>
				
			</div>

		{/if}

  	{:catch error}


  	{/await}

</main>

<style>

.scrollable {
	/* scrollbar-color: hsl(330, 59%, 58%) #171717 !important; */
	scrollbar-width: thin;
	scroll-margin: 2px, 2px, 2px, 2px;
}
.scrollable::-webkit-scrollbar-thumb  {
	background-color:  hsl(330, 59%, 58%);
	border-radius: 5px;
	width: 1px;
	display: block;
	
}
.scrollable::-webkit-scrollbar-track {
	/* background-color:  hsl(330, 59%, 58%); */

}
.scrollable::-webkit-scrollbar {
	background-color: transparent;
	width: 3px;

}
.scrollable::-webkit-scrollbar-track:hover, .scrollable::-webkit-scrollbar:hover {
	width: 6px;

}

</style>