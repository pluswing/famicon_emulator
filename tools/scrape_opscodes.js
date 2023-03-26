import axios from "axios"
import jsdom from "jsdom"
import * as fs from "node:fs"
import * as path from "node:path"
import * as url from "node:url";

const __dirname = url.fileURLToPath(new URL('.', import.meta.url));

const zip = (a, b, c) => {
    const res = []
    for (let i = 0; i < a.length; i++) {
        res.push([a[i], b[i], c[i]])
    }
    return res
}

const indent = (code, n) => {
    const space = Array.from({length: n}, (_, k) => k).map(() => "  ").join("")
    return code.split("\n").map(l => `${space}${l}`).join("\n")
}

const main = async () => {
    const res = await axios.get("https://www.nesdev.org/obelisk-6502-guide/reference.html")
    const dom = new jsdom.JSDOM(res.data)

    const opsNames = Array.from(dom.window.document.querySelectorAll("h3 > a")).map((o) => {
        return o.getAttribute("name")
    })
    // console.log(opsNames)

    const tables = Array.from(dom.window.document.querySelectorAll("table")).splice(1)
    const psTables = tables.filter((_, i) => i % 2 == 0)
    const amTables = tables.filter((_, i) => i % 2 == 1)

    const psEffects = psTables.map((t) => {
        return Array.from(t.querySelectorAll("tr")).reduce((p, tr) => {
            const items = tr.querySelectorAll("td")
            const flag = items[0].children[0].innerHTML.trim()
            const effect = items[2].innerHTML.trim()
            p[flag] = effect
            return p
        }, {})
    })

    // console.log(psEffects)

    const amVariations = amTables.map((t) => {
        return Array.from(t.querySelectorAll("tr")).splice(1).reduce((p, tr) => {
            const tds = tr.querySelectorAll("td")
            const mode = tds[0].children[0].innerHTML.trim().split("\n").map((l) => l.trim()).join(" ").replace(/\(/g, "").replace(/\)/g, "").replace(/ /g, "").replace(/,/g, "_")
            const opcode = tds[1].textContent.trim().replace("$", "")
            const bytes = tds[2].textContent.trim()
            const cyclesRaw = tds[3].textContent.trim()
            const cycles = cyclesRaw.split(" ")[0]
            const cyclesComment = cyclesRaw.split(" ").splice(1).join(" ").split("\n").map((l) => l.trim()).join(" ")
            p.push({mode, opcode, bytes, cycles, cyclesComment})
            return p
        }, [])
    })

    // console.log(amVariations)

    const opcodes = zip(opsNames, psEffects, amVariations).map(([name, effects, variations]) => {
        return variations.map((v) => {
            const comment = v.cyclesComment ? ` /* ${v.cyclesComment} */` : ""
            return `OpCode::new(0x${v.opcode}, "${name}", ${v.bytes}, ${v.cycles}${comment}, AddressingMode::${v.mode}),`
        })
    }).flat().join("\n")

    const unofficialOps = [
      `OpCode::new(0x04, "*NOP", 2, 2, AddressingMode::ZeroPage),`,
      `OpCode::new(0x44, "*NOP", 2, 2, AddressingMode::ZeroPage),`,
      `OpCode::new(0x64, "*NOP", 2, 2, AddressingMode::ZeroPage),`,
      `OpCode::new(0x0C, "*NOP", 3, 2, AddressingMode::Absolute),`,
      `OpCode::new(0x14, "*NOP", 2, 2, AddressingMode::ZeroPage_X),`,
      `OpCode::new(0x34, "*NOP", 2, 2, AddressingMode::ZeroPage_X),`,
      `OpCode::new(0x54, "*NOP", 2, 2, AddressingMode::ZeroPage_X),`,
      `OpCode::new(0x74, "*NOP", 2, 2, AddressingMode::ZeroPage_X),`,
      `OpCode::new(0xD4, "*NOP", 2, 2, AddressingMode::ZeroPage_X),`,
      `OpCode::new(0xF4, "*NOP", 2, 2, AddressingMode::ZeroPage_X),`,
      `OpCode::new(0x1A, "*NOP", 1, 2, AddressingMode::Implied),`,
    ].join("\n")

    const header = `
use crate::cpu::AddressingMode;
use crate::cpu::OpCode;
use crate::cpu::CPU;
`.trim()

    const code = `
lazy_static! {
  pub static ref CPU_OPS_CODES: Vec<OpCode> = vec![
${indent(opcodes, 2)}

${indent(unofficialOps, 2)}
  ];
}
`
    const callCode = `
pub fn call(cpu: &mut CPU, op: &OpCode) {
  match op.name.replace("*", "").as_str() {
${opsNames.map((name) => `
    "${name}" => {
      cpu.${name.toLowerCase()}(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }
`).join("")}
    _ => {
        todo!()
    }
  }
}`

    fs.writeFileSync(path.join(__dirname, "..", "src", "opscodes.rs"), `${header}\n${code}\n${callCode}`)
    console.log("done.")
}

await main()
