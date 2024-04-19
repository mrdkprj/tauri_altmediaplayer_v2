import "./player.css"
import "../common.css";
import Player from "./Player.svelte"

const app = new Player({
  target: document.body,
})

export default app