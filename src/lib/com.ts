import { invoke } from "@tauri-apps/api";

export let data = {};
export const ready = (async () => {
	await invoke("init");
	data = await invoke("get_data");
})();

// rome-ignore lint/suspicious/noExplicitAny: <explanation>
export function iv2(fn:string, data: {[x:string]:any}) {
	return ready.then(_=>{
		
		return invoke(fn, data);
	},err => {
		console.log(err);
	});
}