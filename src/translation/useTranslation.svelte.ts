import { en } from "./en";
import { ja } from "./ja";

type Local = {
    lang: Mp.Lang;
};

export const locale = $state<Local>({ lang: "en" });

export const t = (key: keyof Mp.Labels) => {
    if (locale.lang == "ja") {
        return ja[key];
    } else {
        return en[key];
    }
};
