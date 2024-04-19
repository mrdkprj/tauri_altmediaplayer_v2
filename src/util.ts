import { dirname, basename } from "@tauri-apps/api/path";
import { convertFileSrc } from "@tauri-apps/api/core";
import { stat } from "@tauri-apps/plugin-fs"
import { message } from "@tauri-apps/plugin-dialog";

class Util{

    extname = (name:string | undefined) => {

        if(!name) return "";

        if(name.lastIndexOf(".") < 0) return "";

        return name.substring(name.lastIndexOf(".")).toLowerCase()
    }

    async toFile(fullPath:string):Promise<Mp.MediaFile>{

        const statInfo = await stat(fullPath);
        const dir = await dirname(fullPath)
        const name = await basename(fullPath)

        return {
            id: crypto.randomUUID(),
            fullPath,
            dir,
            src: convertFileSrc(fullPath),
            name:decodeURIComponent(encodeURIComponent(name)),
            date:statInfo.mtime ? statInfo.mtime.getTime() : new Date().getTime(),
            extension:this.extname(fullPath),
        }
    }

    async updateFile(fullPath:string, currentFile:Mp.MediaFile):Promise<Mp.MediaFile>{

        const dir = await dirname(fullPath)
        const name = await basename(fullPath)

        return {
            id: currentFile.id,
            fullPath,
            dir,
            src: convertFileSrc(fullPath),
            name:decodeURIComponent(encodeURIComponent(name)),
            date:currentFile.date,
            extension:currentFile.extension,
        }
    }

    shuffle(targets:any[]){

        const result = [];
        let size = 0;
        let randomIndex = 0;

        while (targets.length > 0) {
            size = targets.length;
            randomIndex = Math.floor(Math.random() * size);

            result.push(targets[randomIndex]);
            targets.splice(randomIndex, 1);
        }

        return result;
    }

    private localCompareName(a:Mp.MediaFile, b:Mp.MediaFile){
        return a.name.replace(this.extname(a.name),"").localeCompare(b.name.replace(this.extname(a.name),""))
    }

    sort(files:Mp.MediaFile[], sortOrder:Mp.SortOrder){

        if(!files.length) return;

        switch(sortOrder){
            case "NameAsc":
                return files.sort((a,b) => this.localCompareName(a,b))
            case "NameDesc":
                return files.sort((a,b) => this.localCompareName(b,a))
            case "DateAsc":
                return files.sort((a,b) => a.date - b.date || this.localCompareName(a,b))
            case "DateDesc":
                return files.sort((a,b) => b.date - a.date || this.localCompareName(a,b))
        }

    }

    groupBy<T>(items:T[], key:keyof T){

        return items.reduce<{ [groupKey:string] : T[]}>((acc, current) => {
              (acc[current[key] as unknown as string] = acc[current[key] as unknown as string] || []).push(current);
              return acc;
        }, {});

    }

    sortByGroup(files:Mp.MediaFile[], sortOrder:Mp.SortOrder){

        if(!files.length) return;

        const groups = this.groupBy(files, "dir")

        const result = Object.values(groups).map(group => this.sort(group, sortOrder)).flat() as Mp.MediaFile[];
        files.length = 0;
        files.push(...result)

    }

    async showErrorMessage(ex:any){
        const mgs = ex.message ? ex.message : ex;
        await message(mgs, {kind:"error"})
    }
}

const util = new Util();

export default util