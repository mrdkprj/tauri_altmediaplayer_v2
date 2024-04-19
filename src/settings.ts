const defaultSettings :Mp.Settings = {
    bounds: {width:1200, height:800, x:0, y:0},
    playlistBounds: {width:400, height:700, x:0, y:0},
    isMaximized: false,
    playlistVisible:true,
    theme:"dark",
    sort:{
        order:"NameAsc",
        groupBy:false,
    },
    video:{
        playbackSpeed:1,
        seekSpeed:10,
        fitToWindow: true,
    },
    audio:{
        volume: 1,
        ampLevel: 0.07,
        mute:false,
    },
    defaultPath:"",
    locale:{
        mode:"system",
        lang:"en"
    },
    tags:[],
}

class Settings{

    data = defaultSettings;
    private ready = false;

    init(settings:Mp.Settings){
        if(this.ready) return this.data;

        this.ready = true;

        return this.createSettings(settings)
    }

    // private setLanguage(langs:string[]){

    //     if(this.data.locale.mode == "system"){

    //         if(langs[0].includes("ja")){
    //             this.data.locale.lang = "ja"
    //         }else{
    //             this.data.locale.lang = "en"
    //         }

    //     }else{

    //         this.data.locale.lang = this.data.locale.mode
    //     }

    // }

    private createSettings(rawSettings:any):Mp.Settings{

        const config = {...defaultSettings} as any;

        Object.keys(rawSettings).forEach(key => {

            if(!(key in config)) return;

            const value = rawSettings[key];

            if(typeof value === "object" && !Array.isArray(value)){

                Object.keys(value).forEach(valueKey => {
                    if(valueKey in config[key]){
                        config[key][valueKey] = value[valueKey]
                    }
                })
            }else{
                config[key] = value;
            }
        })

        return config;
    }

}

const settings = new Settings();
export default settings
