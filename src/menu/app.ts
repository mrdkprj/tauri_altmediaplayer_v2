import "./contextmenu.css"
import "../common.css";
import Menu from "./Menus.svelte"

const app = new Menu({
  target: document.body,
})

export default app