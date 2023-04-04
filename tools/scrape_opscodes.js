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
  const lines = res.data.split("\n")
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
  const codes = []
  let found = false
  let l = []
  let op = ""
  const ops = [];
  for (let i = 0; i < lines.length; i++) {
    if (lines[i] === "=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D") {
      if (found && op) {
        const s = l.findIndex((v) => v.startsWith("------------"))
        l = l.slice(s + 1)
        const e = l.findIndex((v) => v === "")
        l = l.slice(0, e)
        const m = op.match(/\((.+)\)/)
        const name = m[1];
        codes.push(l.map((v) => {
          const s = v.split("|").map((v) => v.trim())
          return [s[0], s[2], s[3], s[4]]
        }).map((l) => {
          const mode = keyMap[l[0]]
          const code = l[1].replace("$", "")
          const bytes = l[2]
          const comment = l[3].match(/\*/) ? " /* (+ some cycles) */" : ""
          const cycles = l[3].replace(/ |\*/g, "").replace("-", "0")
          return `OpCode::new(0x${code}, "*${name}", ${bytes}, ${cycles}${comment}, AddressingMode::${mode}),`
        }));
        ops.push(name)
      }
      op = lines[i - 1]
      l = [];
      found = true;
    }
    if (found) {
      l.push(lines[i])
    }
  }
  return [ops, codes.flat()];
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
      const comment = v.cyclesComment ? ` /* ${v.cyclesComment} */` : ""
      return `OpCode::new(0x${v.opcode}, "${name}", ${v.bytes}, ${v.cycles}${comment}, AddressingMode::${v.mode}),`
    })
  }).flat().join("\n")

  const [onofficialNames, unofficialOps] = await fetchUnofficialOps();
  const unofficialOpsCode = unofficialOps.join("\n")

  onofficialNames.forEach((name) => {
    if (!opsNames.includes(name)) {
      opsNames.push(name)
    }
  })

  const header = `
use crate::cpu::AddressingMode;
use crate::cpu::OpCode;
use crate::cpu::CPU;
`.trim()

  const code = `
lazy_static! {
  pub static ref CPU_OPS_CODES: Vec<OpCode> = vec![
${indent(opcodes, 2)}

${indent(unofficialOpsCode, 2)}
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
