import { writable, derived } from "svelte/store"
import { en } from "./en";
import { ja } from "./ja";

export const lang = writable<Mp.Lang>("en");

const translate = (lang:Mp.Lang, key:keyof Mp.Labels) => {

    if(lang == "ja"){
        return ja[key]
    }else{
        return en[key]
    }
}

export const t = derived(lang, ($lang) => (key:keyof Mp.Labels) => translate($lang, key));