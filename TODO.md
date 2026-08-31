# normordis-pdf — Sessão seguinte

> Contexto para Claude retomar sem re-leitura do histórico.

---

## Estado actual do projecto (2026-06-29)

Crate Rust em `/mnt/c/Users/carlo/Documents/Projetos/normordis-pdf/`.

Features opcionais implementadas e testadas (suite verde com todas activas):
```
cargo test --features "tsa system-fonts optimal_wrap hyphenation"
```

| Feature | Implementação | Estado |
|---|---|---|
| `system-fonts` | `FontRegistry::load_system_fonts()` + `from_system()` + `DocumentBuilder::fonts_from_system()` | ✅ 100% |
| `hyphenation` | Integrado no greedy line-breaker em `layout/engine.rs` | ✅ |
| `optimal_wrap` | `Paragraph.line_breaking: LineBreakingMode` → `layout_runs_with_mode()` | ✅ |
| `tsa` | `src/tsa.rs` — RFC 3161, DER rebuild, `embed_timestamp` / `timestamp_pkcs7` | ✅ 75% (ver abaixo) |

---

## Itens pendentes

### 1. TSA RFC 3161 — merge de unsigned attributes (alta prioridade)

**Ficheiro:** `src/tsa.rs`

**Problema:** `embed_timestamp()` substitui **toda** a `[1] unsignedAttrs` existente em vez de fazer merge.
Para assinaturas qualificadas eIDAS, o PKCS#7 produzido por um HSM já pode conter outros unsigned attributes (ex: `id-smime-aa-signingCertificateV2` OID `1.2.840.113549.1.9.16.2.47`). A substituição total destrói esses atributos.

**Localização do código relevante em `embed_timestamp`:**
```rust
let (insert_pos, remove_end) = if data.get(pos).copied() == Some(0xA1) {
    let existing = nav_el!(data, pos, "SignerInfo: malformed [1] unsignedAttrs")?;
    (pos, existing.full_end())   // ← aqui: remove tudo e substitui
} else {
    (pos, pos)
};
```

**O que fazer:**
- Se `[1]` já existe: parsear os `Attribute` individuais dentro do SET, adicionar o novo TST como elemento extra, reconstruir o SET
- A função auxiliar `parse_attributes(data: &[u8], a1_el: &El) -> Vec<Vec<u8>>` devolve cada `Attribute` como raw bytes; depois `der_wrap(0xA1, &[existing_attrs, &[new_tst_attr]].concat())`
- Adicionar teste: `embed_timestamp_merges_with_existing_attribute`

---

### 2. TSA — teste de integração com TSR gravado (cassette)

**Problema:** O único teste de integração real é `#[ignore]` (requer rede). Não existe teste offline que cubra o path `request_timestamp → extract_timestamp_token → embed_timestamp` com bytes reais.

**O que fazer:**
1. Obter um TSR real de `https://freetsa.org/tsr` para um payload de teste
2. Guardar os bytes em `tests/fixtures/freetsa_response.bin` (ou como `include_bytes!` inline no teste)
3. Escrever teste `tsa_extract_token_from_real_tsr` que usa esses bytes fixos — não precisa de rede

---

### 3. veraPDF CI — RESOLVIDO

**Ficheiro:** `.github/workflows/verapdf.yml`

As dúvidas levantadas aqui foram verificadas contra o instalador real do
veraPDF 1.30.2, num sistema com Java. Resultados:

| Dúvida | Resposta verificada |
|---|---|
| Origem do download | **Não existe** nas releases do GitHub — `veraPDF-apps` só publica tags de build. A origem oficial é `software.verapdf.org/releases/<serie>/`. Era esta a causa do exit 8. |
| Nome do JAR | `verapdf-izpack-installer-1.30.2.jar`, dentro de `verapdf-greenfield-1.30.2/`. O workflow assumia `verapdf-greenfield-1.30.2-installer.jar`. |
| `PacksPanel` | Necessário. A ordem real é `HTMLHelloPanel → TargetPanel → PacksPanel → InstallPanel → FinishPanel`; não existe painel de licença. |
| Instalação por XML automatizado | **Não funciona.** O helper de automação aborta com `[ Automated installation FAILED! ]` sem diagnóstico, mesmo com os painéis corretos. Substituído por modo consola com respostas por `stdin`. |
| Localização do executável | Raiz da instalação (`$VERAPDF_HOME/verapdf`), **não** em `bin/`. A suposição registada aqui estava errada. |

O passo de instalação passou a fixar o ficheiro por `sha256` e a verificar
`test -x "$VERAPDF_HOME/verapdf"` antes de o pôr no PATH, para falhar cedo e
com diagnóstico em vez de falhar mais à frente.

A sequência de respostas do modo consola está documentada em comentário no
próprio workflow. Ao subir de versão, reconfirmar essa sequência: um prompt
novo dessincroniza as respostas e o instalador termina com "You have not
selected any packs!" — que é sucesso aparente seguido de falha.

### 4. Clippy pre-existente (baixa prioridade)

`cargo clippy --workspace -- -D warnings` tem ~40 erros em ficheiros que **não foram tocados hoje**:
- `src/layout/knuth_plass.rs:5` — empty line after doc comment
- `src/backend/pdf_writer_backend.rs` — useless-conversion, too-many-arguments
- `src/elements/toc.rs` — collapsible-if, repeat().take()
- Vários outros — explicit lifetimes, map_or, etc.

Estes **existiam antes das sessões de hoje** e não são regressões. Devem ser tratados numa PR separada.

---

### 5. generate_ndt_schema! proc macro (futuro)

Macro de compile-time que detecta `{{placeholders}}` num `.dotx` e gera struct Rust type-safe. Nenhum código escrito ainda.

---

## Arquitectura relevante para retomar

### `src/tsa.rs` — estruturas chave
```
El { tag, tag_pos, content_start, content_len }
    ↑ sem len_pos/len_bytes (removidos — abordagem rebuild não precisa)

NavResult { oid_el, sd, si_set, si, sig_el, insert_pos, remove_end }
    ↑ sem exp0 (variável local em navigate_pkcs7, não retornada)

navigate_pkcs7(data) → NavResult  (navegação DER até signature OCTET STRING)
embed_timestamp(pkcs7, tst) → Vec<u8>  (rebuild inside-out, sem patch-in-place)
```

### OIDs relevantes (DER sem tag/len)
```rust
// SHA-256: 2.16.840.1.101.3.4.2.1
const SHA256_OID: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01];
// signedData: 1.2.840.113549.1.7.2
const SIGNED_DATA_OID: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x02];
// id-aa-signatureTimeStampToken: 1.2.840.113549.1.9.16.2.14
const TST_OID: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x10, 0x02, 0x0E];
// id-smime-aa-signingCertificateV2: 1.2.840.113549.1.9.16.2.47 (para merge)
```

### Fluxo TSA completo
```
PreparedPdf
  → bytes_to_sign()
  → [HSM] → pkcs7_der (PKCS#7 sem timestamp)
  → timestamp_pkcs7(pkcs7_der, "https://freetsa.org/tsr")  [feature tsa]
       ├─ extract_signature_value(pkcs7_der)  → sig_bytes
       ├─ SHA-256(sig_bytes) → hash
       ├─ build_timestamp_request(hash) → TSQ DER
       ├─ HTTP POST → TSR DER
       ├─ extract_timestamp_token(tsr) → TST (ContentInfo) bytes
       └─ embed_timestamp(pkcs7_der, tst) → pkcs7_com_tst
  → PreparedPdf::embed_signature(pkcs7_com_tst)
  → PDF final (CAdES-T)
```
