import { en } from "./en";
import { ja } from "./ja";

export const translation = (lang:Mp.Lang) => {

    const getTranslator = (lang:Mp.Lang) => {

        const translator = (key:keyof Mp.Labels) => {
            if(lang == "ja"){
                return ja[key]
            }else{
                return en[key]
            }
        }

        translator.lang = lang

        return translator;
    }

    return getTranslator(lang);
}