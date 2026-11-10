import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import init from './lib/wasm-pkg/sudoku_wasm'

// O módulo WASM precisa ser baixado e instanciado antes de qualquer `new WebGame`. O `init` é
// assíncrono porque o navegador compila o .wasm enquanto ele chega pela rede.
await init()

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
