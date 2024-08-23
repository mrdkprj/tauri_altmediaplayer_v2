<script lang="ts">

    import { onMount } from "svelte";
    import Footer from "./Footer.svelte";
    import icon from "../assets/icon.ico"

    import { appState, dispatch } from "./appStateReducer";
    import { t, lang } from "../translation/useTranslation"
    import { IPC } from "../ipc";
    import util from "../util";
    import { FORWARD, BACKWARD, APP_NAME, Buttons, handleKeyEvent, PlayableAudioExtentions } from "../constants";
    import { getDropFiles } from "../fileDropHandler";
    import { handleShortcut } from "../shortcut";

    import { appDataDir } from "@tauri-apps/api/path";
    import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { save } from '@tauri-apps/plugin-dialog';
    import { dirname, join } from "@tauri-apps/api/path";
    import { writeFile } from '@tauri-apps/plugin-fs';
    import { open } from "@tauri-apps/plugin-shell";

    const ipc = new IPC("Player");
    let settings:Mp.Settings;
    let video:HTMLVideoElement
    let container:HTMLDivElement
    let hideControlTimeout:number | null
    let afterReleaseCallback:(() => void) | undefined;

    const updateTime = (progress:number) => {

        if(!$appState.loaded) return;

        video.currentTime = $appState.media.videoDuration * progress;

    }

    const onTimeUpdate = () => {

        if(!$appState.loaded) return;

        const duration = $appState.media.videoDuration > 0 ? $appState.media.videoDuration : 1

        dispatch({type:"currentTime", value:video.currentTime})

        ipc.send("progress", {progress:video.currentTime / duration})
    }

    const updateVolume = (volume:number) => {

        if(volume > 1 || volume < 0) return;

        video.volume = volume
        dispatch({type:"videoVolume", value:volume})
    }

    const getGainNode = () => {

        if(!video) throw new Error("Media not found");

        if($appState.media.gainNode) return $appState.media.gainNode;

        const audioCtx = new AudioContext();
        const source = audioCtx.createMediaElementSource(video);
        const gainNode = audioCtx.createGain();

        dispatch({type:"gainNode", value:gainNode})

        source.connect(gainNode);
        gainNode.connect(audioCtx.destination);

        return gainNode;

    }

    const updateAmpLevel = (ampLevel:number) => {

        if(ampLevel > 1 || ampLevel < 0) return;

        const gainNode = getGainNode();

        dispatch({type:"ampLevel", value:ampLevel})

        gainNode.gain.value = ampLevel * 10;

    }

    const toggleMute = () => {
        dispatch({type:"mute", value:!$appState.media.mute})
    }

    const onFileDrop = (e:Mp.FileDropEvent) => {
console.log("drop");
        const files = getDropFiles(e)

        if(files.length){
            ipc.sendTo("Playlist", "load-playlist", {files})
        }
    }

    const initPlayer = () => {
        dispatch({type:"init"})
        video.load();
    }

    const loadMedia = (e:Mp.FileLoadEvent) => {

        dispatch({type:"currentFile", value:e.currentFile})
        dispatch({type:"currentTime", value:0})
        dispatch({type:"startFrom", value:e.startFrom})

        video.autoplay = e.type == "Play" ? true : $appState.playing;
        video.muted = $appState.media.mute;
        video.playbackRate = $appState.media.playbackSpeed

        video.load();

    }

    const onMediaLoaded = () => {

        dispatch({type:"loaded", value:true})

        document.title = `${APP_NAME} - ${$appState.currentFile.name}`

        changeVideoSize();

        dispatch({type:"videoDuration", value:video.duration})

        if($appState.startFrom){
            changeCurrentTime($appState.startFrom)
        }

    }

    const onLoadError = async () => {

        let loaded = $appState.loaded;

        dispatch({type:"loaded", value:false})
        dispatch({type:"playStatus", value:"stopped"})

        document.title = `${APP_NAME} - ${$appState.currentFile.name}`

        dispatch({type:"videoDuration", value:0})

        video.autoplay = false;

        if(loaded){
            await util.showErrorMessage($t("unsupportedMedia"));
        }

    }

    const onEmptied = () => {

        if(!afterReleaseCallback) return

        afterReleaseCallback();
        afterReleaseCallback = undefined;

    }

    const releaseFile = (data:Mp.ReleaseFileRequest) => {

        if(data.fileIds.includes($appState.currentFile.id)){
            const currentTime = $appState.media.currentTime;
            initPlayer();
            afterReleaseCallback = () => ipc.sendTo("Playlist", "file-released", {currentTime})
        }else{
            ipc.sendTo("Playlist", "file-released", {currentTime:0})
        }

    }

    const changeVideoSize = () => {

        const containerRect = container.getBoundingClientRect();

        if($appState.media.fitToWindow && containerRect.height > video.videoHeight){
            const ratio = Math.min(containerRect.width / video.videoWidth, containerRect.height / video.videoHeight);
            video.style.height = `${video.videoHeight * ratio}px`
        }else{
            video.style.height = ""
        }

    }

    const changeCurrentTime = (time:number) => {

        if(!$appState.loaded) return;

        const nextTime = video.currentTime + time;

        if(nextTime >= video.duration){
            return changeFile(FORWARD)
        }

        if(nextTime < 0){
            return changeFile(BACKWARD)
        }

        video.currentTime = nextTime;

    }

    const playFoward = (button:number) => {

        if(!$appState.loaded) return;

        if(button === Buttons.left){
            changeCurrentTime($appState.media.seekSpeed);
        }

        if(button === Buttons.right){
            changeFile(FORWARD)
        }

    }

    const playBackward = (button:number) => {

        if(!$appState.loaded) return;

        if(button === Buttons.left){
            changeCurrentTime(-$appState.media.seekSpeed)
        }

        if(button === Buttons.right){
            changeFile(BACKWARD)
        }

    }

    const changeFile = (index:number) => {
        ipc.sendTo("Playlist", "change-playlist", {index})
    }

    const togglePlay = async () => {

        if(!$appState.loaded) return;

        if(video.paused){
            await video.play();
        }else{
            video.pause();
        }

    }

    const onPlayed = () => {
        ipc.send("play-status-change", {status:"playing"})
        dispatch({type:"playStatus", value:"playing"})
    }

    const onPaused = () => {

        if(video.currentTime == video.duration) return;

        ipc.send("play-status-change", {status:"paused"})
        dispatch({type:"playStatus", value:"paused"})
    }

    const stop = () => {

        if(!$appState.loaded) return;

        ipc.send("play-status-change", {status:"stopped"})
        dispatch({type:"playStatus", value:"stopped"})
        video.load();

    }

    const requestPIP = async () => {
        if($appState.loaded){
            await video.requestPictureInPicture();
        }
    }

    const changePlaybackSpeed = (speed:number) => {
        dispatch({type:"playbackSpeed", value:speed})
        video.playbackRate = speed
    }

    const changeSeekSpeed = (speed:number) => {
        dispatch({type:"seekSpeed", value:speed})
    }

    const captureMedia = async () => {

        if(!$appState.loaded || PlayableAudioExtentions.includes($appState.currentFile.extension)) return;

        const canvas = document.createElement("canvas");
        const rect = video.getBoundingClientRect();
        canvas.width = rect.width;
        canvas.height = rect.height;

        const context = canvas.getContext("2d");
        if(context){
            context.drawImage(video, 0, 0, rect.width, rect.height);
        }
        const data = canvas.toDataURL("image/jpeg").replace(/^data:image\/jpeg;base64,/, "");

        const savePath = await save({
            defaultPath: (await join(settings.defaultPath, `${$appState.currentFile.name}-${video.currentTime}.jpeg`)),
            filters: [
                { name: "Image", extensions: ["jpeg", "jpg"] },
            ],
        })

        if(!savePath) return;

        settings.defaultPath = await dirname(savePath);

        writeFile(savePath, Uint8Array.from(atob(data), c => c.charCodeAt(0)))

        await ipc.send("sync-settings", settings);
    }

    const minimize = async () => {
        await WebviewWindow.getCurrent().minimize()
    }

    const toggleMaximize = async () => {
        dispatch({type:"isMaximized", value:!$appState.isMaximized})
        await WebviewWindow.getCurrent().toggleMaximize()
    }

    const onWindowSizeChanged = (e:Mp.ResizeEvent) => {
        dispatch({type:"isMaximized", value:e.isMaximized})
    }

    const hideControl = () => {
        hideControlTimeout = window.setTimeout(() => {
            dispatch({type:"autohide", value:true})
        },2000)
    }

    const exitFullscreen = async () => {

        dispatch({type:"isFullScreen", value:false})

        if(hideControlTimeout){
            window.clearTimeout(hideControlTimeout)
        }
        dispatch({type:"autohide", value:false})

        await WebviewWindow.getCurrent().setFullscreen(false)
        if(settings.playlistVisible){
            WebviewWindow.getByLabel("Playlist")?.show();
        }
    }

    const enterFullscreen = async () => {
        dispatch({type:"isFullScreen", value:true})
        hideControl()

        await WebviewWindow.getCurrent().setFullscreen(true)
        WebviewWindow.getAll().filter(view => view.label != "Player").forEach(view => view.hide())
    }

    const toggleFullscreen = async () => {
        if($appState.isFullScreen){
            await exitFullscreen()
        }else{
            await enterFullscreen()
        }
    }

    const showControl = () => {

        if(!$appState.isFullScreen) return;

        dispatch({type:"autohide", value:false})
        if(hideControlTimeout){
            window.clearTimeout(hideControlTimeout)
        }

        if(!$appState.preventAutohide){
            hideControl();
        }
    }

    const toggleConvert = () => {
        dispatch({type:"converting"})
    }

    const onChangeDisplayMode = () => {
        const mode = !$appState.media.fitToWindow
        dispatch({type:"fitToWindow", value:mode})
        changeVideoSize();
    }

    const load = (e:Mp.FileLoadEvent) => {
        if(e.currentFile.id){
            loadMedia(e)
        }else{
            initPlayer();
        }
    }

    const onMousemove = () => {
        showControl();
    };

    const onKeydown = (e:KeyboardEvent) => {
        if(e.ctrlKey && e.key === "r") e.preventDefault();

        if(e.key === "F5") return ipc.invoke("restart", undefined);

        if(e.key === "ArrowRight"){

            showControl();

            if(e.shiftKey){
                changeCurrentTime($appState.media.seekSpeed)
            }else{
                playFoward(Buttons.left);
            }

            return
        }

        if(e.key === "ArrowLeft"){

            showControl();

            if(e.shiftKey){
                changeCurrentTime(-$appState.media.seekSpeed)
            }else{
                playBackward(Buttons.left);
            }

            return
        }

        if(e.key === "ArrowUp"){
            showControl();
            updateVolume($appState.media.videoVolume + 0.01)
            return
        }

        if(e.key === "ArrowDown"){
            showControl();
            updateVolume($appState.media.videoVolume - 0.01)
            return
        }

        if(e.key === "Escape"){
            return exitFullscreen();
        }

        if(e.ctrlKey && e.key === "m"){
            return toggleMute();
        }

        if(e.key === "Enter"){
            return togglePlay();
        }

        const shortcut = handleShortcut("Player", e);

        if(shortcut){
            return handleContextMenu(shortcut)
        }
    };

    const onResize = () => {
        changeVideoSize()
    };

    const onContextMenu = (e:MouseEvent) => {
        e.preventDefault()
        e.stopPropagation()
        ipc.invoke("open_context_menu", {x:e.screenX, y:e.screenY})
    }

    const togglePlaylistWindow = async () => {

        const playlist = WebviewWindow.getByLabel("Playlist");

        settings.playlistVisible = !settings.playlistVisible;

        if(settings.playlistVisible){
            await playlist?.show()
        }else{
            await playlist?.hide()
        }

        await ipc.send("sync-settings", settings);
    }

    const changeTheme = async (theme:Mp.Theme) => {
        console.log(theme)
        settings.theme = theme;
        await ipc.invoke("change_theme", theme)
        await ipc.send("sync-settings", settings);
    }

    const showSettingsJson = async () => {
        const dataDir = await appDataDir();
        const fullpath = await join(dataDir, "temp", "altmediaplayer.settings.json")
        await open(fullpath);
    }

    const syncSettings = (newSettings:Mp.Settings) => {
        settings = newSettings;
    }

    const handleContextMenu = (e:Mp.ContextMenuEvent) => {
        const id = e.name ? e.name : e.id;
        switch(id){
            case "PlaybackSpeed":
                changePlaybackSpeed(Number(e.id));
                break;
            case "SeekSpeed":
                changeSeekSpeed(Number(e.id));
                break;
            case "TogglePlaylistWindow":
                togglePlaylistWindow();
                break;
            case "FitToWindow":
                onChangeDisplayMode();
                break;
            case "PictureInPicture":
                requestPIP();
                break;
            case "ToggleFullscreen":
                toggleFullscreen()
                break;
            case "Theme":
                changeTheme(e.id as Mp.Theme);
                break;
            case "Capture":
                captureMedia();
                break;
            case "ViewSettingsJson":
                showSettingsJson();
                break;
        }
    }

    const close = async () => {
/*
settings.audio.mute = $appState.media.mute;
        settings.video.fitToWindow = $appState.media.fitToWindow
        settings.audio.volume = $appState.media.videoVolume;
        settings.audio.ampLevel = $appState.media.ampLevel;
        settings.video.playbackSpeed = $appState.media.playbackSpeed;
        settings.video.seekSpeed = $appState.media.seekSpeed

        await ipc.invoke("save", settings)
*/
        WebviewWindow.getCurrent().close()
    }

    const prepare = async (e:Mp.ReadyEvent) => {
console.log(e)
        settings = e.settings;

        $lang = settings.locale.lang;

        dispatch({type:"isMaximized", value:settings.isMaximized})

        updateVolume(settings.audio.volume);
        updateAmpLevel(settings.audio.ampLevel)

        dispatch({type:"mute", value:settings.audio.mute})

        dispatch({type:"fitToWindow", value:settings.video.fitToWindow})
        dispatch({type:"playbackSpeed", value:settings.video.playbackSpeed})
        dispatch({type:"seekSpeed", value:settings.video.seekSpeed})

        initPlayer();

    }

    onMount(() => {

        ipc.receiveOnce("ready", prepare)
        ipc.receive("sync-settings", syncSettings);
        ipc.receive("load-file", load)
        ipc.receive("contextmenu-event", handleContextMenu)
        ipc.receiveTauri<Mp.FileDropEvent>("tauri://drag-drop", onFileDrop)
        ipc.receive("toggle-play", togglePlay)
        ipc.receive("restart", initPlayer)
        ipc.receive("release-file", releaseFile)
        ipc.receive("after-toggle-maximize", onWindowSizeChanged)
        ipc.receive("toggle-convert", toggleConvert)
        ipc.receive("toggle-fullscreen", toggleFullscreen)
        ipc.receive("log", data => console.log(data.log))

        ipc.invoke("retrieve_settings", undefined);

        return () => {
            ipc.release();
        }
    })
</script>

<svelte:window on:keydown={onKeydown} on:resize={onResize} on:contextmenu={e => e.preventDefault()}/>
<svelte:document on:mousemove={onMousemove}/>

<div class="player-viewport" class:full-screen={$appState.isFullScreen} class:loaded={$appState.loaded} class:autohide={$appState.autohide}>

    <div data-tauri-drag-region class="player-title-bar">
        <div data-tauri-drag-region class="icon-area">
            <img class="ico" src={icon} alt=""/>
            <span>{APP_NAME}</span>
        </div>
        <div data-tauri-drag-region class="title">{$appState.currentFile.name}</div>
        <div class="window-area">
            <div class="minimize" on:click={minimize} on:keydown={handleKeyEvent} role="button" tabindex="-1">&minus;</div>
            <div class="maximize" on:click={toggleMaximize} on:keydown={handleKeyEvent} role="button" tabindex="-1">
                <div class:minbtn={$appState.isMaximized} class:maxbtn={!$appState.isMaximized}></div>
            </div>
            <div class="close" on:click={close} on:keydown={handleKeyEvent} role="button" tabindex="-1">&times;</div>
        </div>
    </div>

    <div bind:this={container} class="video-container" on:dragover={e => e.preventDefault()} on:dblclick={togglePlay} on:contextmenu={onContextMenu} role="button" tabindex="-1">
        <video
            bind:this={video}
            class="video"
            src={$appState.currentFile.src}
            on:loadeddata={onMediaLoaded}
            on:ended={() => changeFile(FORWARD)}
            on:timeupdate={onTimeUpdate}
            on:play={onPlayed}
            on:pause={onPaused}
            on:contextmenu={onContextMenu}
            on:emptied={onEmptied}
            on:error={onLoadError}
            muted={$appState.media.mute}
            crossorigin="anonymous"
        />
    </div>

    <Footer
        onMouseEnter={showControl}
        onUpdateTime={updateTime}
        onUpdateVolume={updateVolume}
        onUpdateAmpLevel={updateAmpLevel}
        onClickPlay={togglePlay}
        onClickStop={stop}
        onClickPrevious={playBackward}
        onClickNext={playFoward}
        onClickMute={toggleMute}
        t={$t}
    />

</div>
