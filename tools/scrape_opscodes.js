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
  const space = Array.from({ length: n }, (_, k) => k).map(() => "  ").join("")
  return code.split("\n").map(l => `${space}${l}`).join("\n")
}

const fetchUnofficialOps = async () => {
  const res = await axios.get("https://www.nesdev.org/undocumented_opcodes.txt")
  const lines = [...res.data.split("\n"), ""]
  const keyMap = {
    "Implied": "Implied",
    "Immediate": "Immediate",
    "Zero Page": "ZeroPage",
    "Zero Page,X": "ZeroPage_X",
    "Zero Page,Y": "ZeroPage_Y",
    "Absolute": "Absolute",
    "Absolute,X": "Absolute_X",
    "Absolute,Y": "Absolute_Y",
    "(Indirect,X)": "Indirect_X",
    "(Indirect),Y": "Indirect_Y",
  }
  const defs = []
  let found = false
  let l = []
  let op = ""
  for (let i = 0; i < lines.length; i++) {
    if (lines[i] === "=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D") {
      if (found && op) {
        const m = op.match(/\((.+)\)/)
        const name = m[1];

        const s = l.findIndex((v) => v.startsWith("------------"))
        l = l.slice(s + 1)
        const e = l.findIndex((v) => v === "")
        l = l.slice(0, e)
        defs.push([name, l])
      }
      op = lines[i - 1]
      l = [];
      found = true;
    }
    if (found) {
      l.push(lines[i])
    }
  }

  const m = op.match(/\((.+)\)/)
  const name = m[1];

  const s = l.findIndex((v) => v.startsWith("------------"))
  l = l.slice(s + 1)
  const e = l.findIndex((v) => v === "")
  l = l.slice(0, e)
  defs.push([name, l])

  const codes = defs.map(([name, lines]) => {
    return lines.map((v) => {
      const s = v.split("|").map((v) => v.trim())
      const mode = keyMap[s[0]]
      const code = s[2].replace("$", "")
      const bytes = s[3]
      let cmode = "None"
      if (s[4].match(/\*/)) {
        cmode = "Page"
      }
      const cycles = s[4].replace(/ |\*/g, "").replace("-", "0")
      return `m.insert(0x${code}, OpCode::new(0x${code}, "*${name}", ${bytes}, ${cycles}, CycleCalcMode::${cmode}, AddressingMode::${mode}));`
    });
  }).flat();
  return [defs.map((v) => v[0]), codes];
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
      p.push({ mode, opcode, bytes, cycles, cyclesComment })
      return p
    }, [])
  })

  // console.log(amVariations)

  const opcodes = zip(opsNames, psEffects, amVariations).map(([name, effects, variations]) => {
    return variations.map((v) => {
      let mode = "None"
      if (v.cyclesComment == "(+1 if page crossed)") {
        mode = "Page"
      } else if (v.cyclesComment) {
        mode = "Branch"
      }
      return `m.insert(0x${v.opcode}, OpCode::new(0x${v.opcode}, "${name}", ${v.bytes}, ${v.cycles}, CycleCalcMode::${mode}, AddressingMode::${v.mode}));`
    })
  }).flat().join("\n")

  const [unofficialNames, unofficialOps] = await fetchUnofficialOps();
  const unofficialOpsCode = unofficialOps.join("\n")

  unofficialNames.forEach((name) => {
    if (!opsNames.includes(name)) {
      opsNames.push(name)
    }
  })

  const header = `
use std::{collections::HashMap, sync::Mutex};
use once_cell::sync::Lazy;
use crate::cpu::{AddressingMode, CycleCalcMode, OpCode, CPU};
`.trim()

  const code = `
pub static CPU_OPS_CODES: Lazy<HashMap<u8, OpCode>> = Lazy::new(|| {
  let mut m = HashMap::new();

${indent(opcodes, 1)}

${indent(unofficialOpsCode, 1)}
  m
});
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
