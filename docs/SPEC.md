# EnzoBallZ — Especificação do MVP

> Spec fechado a partir da entrevista `@grillme`. Valores numéricos são o ponto
> de partida; o balanceamento pode mudar sem alterar a arquitetura.
>
> Este documento descreve o **MVP atual**. A evolução planejada para 4 slots
> genéricos de habilidade, skills modulares, remapeamento de teclas e controles
> mobile está em [`ACTIONS_AND_SKILLS.md`](ACTIONS_AND_SKILLS.md). Onde este
> documento e o plano divergirem, o texto marcado como “atual” vale para o
> código de hoje e o plano vale para o alvo.

## 1. Objetivo

Um jogo de plataforma 2D estilo Mario, em Rust compilado para WebAssembly, onde
o Enzo corre, pula, luta e farma aura alternando as teclas `6` e `7`. A aura é
mana para o golpe especial **Mogging**. Inimigos dão XP; XP sobe de nível e concede
pontos para distribuir entre Velocidade, Força, Vida e Aura.

## 2. Escopo

- Uma fase com scroll horizontal e tilemap.
- Um jogador no MVP, com core pronto para N jogadores.
- Movimento estilo Mario, incluindo corrida por double-tap.
- Farming de aura com `6`/`7` alternados, combo e lentidão.
- Combo físico de 3 golpes.
- Um golpe especial: **Mogging** (projétil instantâneo).
- Um tipo de inimigo patrulhador.
- XP, level-up, pontos e quatro atributos.
- Vida, aura, morte com respawn e vitória por bandeira.
- Menu inicial com seleção de três slots de save e opções de idioma.
- HUD fixa com janelas arrastáveis de status, inventário, equipamento e
  habilidades, abertas por dots.
- Interface em pt-BR e inglês, com escolha persistida.
- Auto-save local da progressão por slot.
- Sprites pixel-art originais gerados por script.

## 3. Não-objetivos do MVP atual

- Multiplayer em rede, servidor, matchmaking, autenticação.
- Co-op local.
- Mais de uma fase, chefes, loja e coleta de itens no mapa.
- Áudio.
- Opções gráficas.
- Assets de terceiros (incluindo personagens de Dragon Ball ou Mario).

Os itens abaixo saíram do não-objetivo e passaram a ser **evolução planejada**,
não implementada no código atual:

- Catálogo amplo de skills além do Mogging (voo, tiro de nutela, grito etc.), com
  troca de skill equipada em 4 slots genéricos.
- Remapeamento de teclas e controles mobile/touch.
- HUD e menus em HTML por cima do canvas.

## 4. Loop de jogo

```text
Explorar e enfrentar inimigos
        │
        ├─ ataque físico (J) ──▶ dano grátis, curto alcance
        │
        ├─ farmar aura (6/7) ──▶ enche a barra, mas deixa o Enzo lento
        │
        └─ Mogging (K) ─────────▶ gasta aura, dano à distância
                                  │
                                  ▼
                         inimigo morre ──▶ XP
                                  │
                                  ▼
                         level-up ──▶ +1 ponto
                                  │
                                  ▼
                 distribuir: Velocidade / Força / Vida / Aura
                                  │
                                  ▼
                         chegar à bandeira = vitória
```

## 5. Controles

| Ação | Teclas | Observação |
| ---- | ------ | ---------- |
| Mover | `A`/`D` ou `←`/`→` | analógico digital |
| Pular | `Espaço` ou `W`/`↑` | edge-triggered |
| Correr | double-tap na direção | segurar a direção mantém a corrida |
| Dash | `Shift` | impulso horizontal curto, com cooldown |
| Ataque físico | `J` | combo de 3 |
| Mogging | `K` | custa aura |
| Farmar aura | `6` e `7` | alternância obrigatória |
| Distribuir ponto | `1` `2` `3` `4` ou clique | na janela de status |
| Confirmar | `Enter` ou clique | fecha a janela de level-up |
| Reiniciar fase | `R` ou clique | na tela de morte/vitória |
| Abrir menu | `Esc` | salva e volta ao menu inicial |
| Navegar menus/HUD | mouse | clique edge-triggered |

### 5.1 Alvo: 4 action slots (planejado)

O alvo substitui os campos fixos de input (`jump`, `dash`, `melee`, `mogging`) por
quatro slots genéricos. Qualquer skill disponível pode ocupar qualquer slot, e a
mesma skill nunca pode estar em dois slots.

| Slot | Tecla padrão | Skill padrão |
| ---- | ------------ | ------------ |
| 1 | `Espaço` | Pulo |
| 2 | `Shift` | Dash |
| 3 | `J` | Soco (combo físico) |
| 4 | `K` | Mogging |

- Movimento: `A`/`D` ou `←`/`→`; vertical: `W`/`S` ou `↑`/`↓`.
- `W`/`↑` deixam de ser pulo no alvo: viram a ação semântica “subir”, usada pelo
  voo. O pulo pertence ao slot 1.
- `1`–`4` continuam distribuindo atributos na tela de level-up.
- SixSeven continua em `6`/`7`, **fora** dos slots.
- O loadout (`slot → skill`) é regra do core, persiste no save e aparece no
  snapshot. Trocar skill é feito no menu de habilidades, com o jogo pausado.
- Skills declaram requisitos (personagem, nível, atributos, item equipado) e têm
  ícone próprio.

Contratos, UX, assets, migração e riscos estão em
[`ACTIONS_AND_SKILLS.md`](ACTIONS_AND_SKILLS.md).

## 6. Movimento

- Chão: aceleração `1800 px/s²`, desaceleração `2200 px/s²`.
- Velocidade de caminhada: `220 px/s`; de corrida: `340 px/s`.
- Ar: aceleração `1200 px/s²`, sem atrito.
- Pulo: impulso `-560 px/s`; gravidade `1800 px/s²`; queda máxima `900 px/s`.
- **Coyote time:** `0,10 s` após sair da plataforma.
- **Jump buffer:** `0,12 s` antes de tocar o chão.
- **Double-tap:** duas pressões na mesma direção em até `0,25 s` ativam a
  corrida. Soltar a direção, inverter ou parar cancela.
- Pulo variável: soltar o botão cedo corta a subida pela metade.
- **Dash:** impulso de `520 px/s` por `0,14 s`, cooldown de `0,7 s`, sem custo
  de aura. Usa a direção segurada ou o facing atual; pode ser usado no chão e no
  ar.
- **Voo:** componente `Fly` existe para inimigos/itens futuros. Não nasce
  atribuído ao jogador: exige nível 5 e 5 pontos em Aura.
- Colisão AABB contra tiles; resolução em X e depois em Y.

Cada movimento é um componente próprio (`Walk`, `Jump`, `Dash`, `Fly`,
`SixSeven`) dentro de `Movement`; o controlador só define intenção. Todo
movimento declara requisitos (tipos de entidade, nível e atributos mínimos) e só
entra no `MoveSet` da entidade quando eles são cumpridos.

> **Alvo (não implementado):** andar e SixSeven continuam mecânicas fora dos
> slots; pulo, dash, voo, soco e Mogging viram `SkillId` equipáveis em
> `Loadout`. `Movement`/`Combat` dão lugar a `Locomotion`/`Skills`. Ver
> [`ACTIONS_AND_SKILLS.md`](ACTIONS_AND_SKILLS.md).

## 7. Aura e SixSeven

- Barra de aura base: `100`.
- **SixSeven** é o movimento de farming: alternar `6 → 7 → 6 → 7` conta como
  acerto. O componente vive em `movements/sixseven.rs`.
- Repetir a mesma tecla (ex.: `6, 6`) zera o combo e não gera aura.
- Cada acerto gera `2` de aura × multiplicador.
- Multiplicador: `1 + 0,15 × combo`, limitado a `3,0`.
- Janela do combo: `0,9 s` sem acerto → combo volta a zero.
- Enquanto o combo está ativo, a velocidade do Enzo cai para `45%`.
- A aura não regenera sozinha. Só o SixSeven recompõe.
- A aura é gasta apenas no Mogging.

## 8. Combate

### 8.1 Combo físico (`J`)

- Três golpes encadeados dentro de `0,5 s` entre um e outro.
- Dano base: `6`, `6`, `12` (+ bônus de Força).
- Alcance: `34 px` na direção em que o Enzo olha, altura `26 px`.
- O terceiro golpe aplica knockback de `220 px/s`.
- Sem custo de aura.
- Se a janela expirar, o combo reinicia no primeiro golpe.
- O Enzo pode andar durante o ataque.

### 8.2 Mogging (`K`)

- Projétil instantâneo, horizontal, na direção em que o Enzo olha.
- Custo: `25` de aura.
- Dano: `20` + `2 × pontos em Aura`.
- Velocidade: `620 px/s`; duração: `1,4 s`.
- Atravessa inimigos (cada inimigo leva dano uma vez por projétil).
- Desaparece ao bater em tile sólido.
- Cooldown: `0,35 s`.
- Pode ser usado andando, correndo ou no ar.
- Sem aura suficiente: não dispara (feedback visual na barra).

## 9. Inimigo: Capanga

- Vida: `30`.
- Dano de contato: `10`, com `1,0 s` de invulnerabilidade no jogador.
- Patrulha a `60 px/s`, vira em paredes e beiradas.
- XP ao morrer: `25`.
- Knockback ao levar dano.
- Todos os inimigos renascem quando o jogador morre.
- Sem respawn por tempo no MVP.

### 9.1 Modelo comum de entidades

Jogador, Capanga e projétil Mogging são instâncias do mesmo `Entity`, com os
mesmos componentes de base:

| Componente | Jogador | Capanga | Mogging |
| ---------- | ------- | ------- | ----- |
| `Body` (posição, velocidade, tamanho) | sim | sim | sim |
| `Health` (vida atual, i-frames) | sim | sim | 1 de vida |
| `Stats` (velocidade, força, vida máx., aura máx.) | derivado dos atributos | fixo | zerado |
| `TeamId` | `PLAYERS` | `ENEMIES` | `PLAYERS` |
| `Controller` | `Player(PlayerController)` | `Ai(SlimeAi)` | `None` |
| `Movement` (`Walk`/`Jump`/`Dash`/`Fly`/`SixSeven`) | andar, pular, dash, SixSeven | andar | nenhum |
| `Combat` (`Melee`/`Mogging`) | combo e Mogging | nenhum | nenhum |
| `MoveSet` | movimentos atribuídos e validados | idem | vazio |
| Dados específicos | nível, XP, pontos, aura | patrulha, XP | dono, dano |

Cada entidade concreta tem seu arquivo (`entities/enzo.rs`, `slime.rs`,
`mogging.rs` e `item/`). Os módulos genéricos guardam só componentes e contratos.

A criação é centralizada em `EntityStore::spawn(EntitySpec)`. Nenhum sistema
monta uma entidade à mão nem duplica atributos. Movimentos e ataques só são
atribuídos se cumprirem os requisitos declarados por eles. O controlador é um
componente: o jogador guarda o `PlayerInput` do tick; a IA guarda o próprio
estado, sem campos paralelos no `Entity`.

## 10. Progressão

- XP necessário para o próximo nível: `100 × nível atual`.
  - Nível 1 → 2: 100 XP (4 Capangas).
  - Nível 2 → 3: 200 XP (8 Capangas).
- Cada level-up:
  - concede `1` ponto de atributo;
  - restaura vida e aura ao máximo;
  - pausa a simulação e abre a tela de distribuição.
- Atributos iniciais: `0` ponto em cada.
- Efeitos por ponto:

| Atributo | Efeito por ponto |
| -------- | ---------------- |
| Velocidade | `+10 px/s` caminhada e `+6 px/s` corrida |
| Força | `+2` de dano em cada golpe do combo físico |
| Vida | `+10` de vida máxima (e cura `+10`) |
| Aura | `+10` de aura máxima e `+2` de dano no Mogging |

- A tela de level-up mostra os quatro atributos e permite gastar vários pontos
  antes de confirmar com `Enter`.

## 11. Vida, morte e vitória

- Vida base: `100`; aura base: `100`.
- Dano de contato zera a vida → fase `Dead` por `1,2 s`.
- Respawn no início da fase com vida/aura cheias.
- XP, nível, pontos e atributos são mantidos.
- Combo de aura e projéteis são resetados.
- Cair fora do mapa conta como morte.
- Tocar a bandeira no fim da fase → fase `Victory`, save gravado e overlay.
- `R` reinicia a fase mantendo a progressão.

## 12. Save

- Três slots independentes no `localStorage`:
  `enzoballz.save.slot.0.v1`, `enzoballz.save.slot.1.v1`,
  `enzoballz.save.slot.2.v1`.
- A chave antiga `enzoballz.save.v1` é lida como fallback do slot 0.
- Formato:

```json
{
  "version": 1,
  "level": 1,
  "xp": 0,
  "points": 0,
  "attributes": { "speed": 0, "strength": 0, "health": 0, "aura": 0 }
}
```

- Gravado ao subir de nível, ao confirmar a distribuição, ao morrer, ao vencer e
  ao voltar para o menu.
- Posição, inimigos e projéteis não são salvos.
- O idioma escolhido fica em `enzoballz.language` e é restaurado ao abrir o jogo.
- Versão incompatível → save ignorado com aviso no console (sem crash).
- Apagar um slot remove apenas a chave daquele slot.

> **Alvo (não implementado):** o save sobe para `SAVE_VERSION = 2` com
> `loadout: [Option<SkillId>; 4]`; saves v1 são migrados preenchendo o loadout
> padrão. As chaves passam a `enzoballz.save.slot.{0..2}.v2`. Detalhes em
> [`ACTIONS_AND_SKILLS.md`](ACTIONS_AND_SKILLS.md).

## 13. HUD e UI

- Menu inicial com as telas: principal, saves (3 slots) e opções (idioma).
- A tela de saves permite iniciar um jogo novo, carregar ou apagar um slot; a
  exclusão pede confirmação em um modal.
- HUD fixa com barra de vida (vermelha), aura (ciano), XP (amarela), nível,
  pontos, Mogging e o botão de menu.
- Quatro dots abrem/fecham janelas arrastáveis: status, inventário, equipamento
  e habilidades. O dot de status pisca enquanto houver level-up pendente.
- As janelas são arrastáveis pelo título, têm foco/empilhamento e botão de
  fechar; a posição é mantida durante a sessão.
- A janela de status concentra a distribuição de pontos: clique em um atributo
  ou use `1`–`4`, e confirme com `Enter` ou pelo botão.
- Dica de controles no início da fase, banners de fase/level-up e overlays de
  morte e vitória com botões de reiniciar e voltar ao menu.
- Todos os textos vêm de um sistema de i18n com pt-BR e inglês.

> **Alvo (não implementado):** a barra fixa troca o indicador único de Mogging por
> uma action bar com os 4 slots (ícone, tecla, custo, cooldown e bloqueio). A
> janela de habilidades vira o editor de loadout: clicar no slot e depois na
> skill, com troca automática se a skill já estiver em outro slot. O menu abre
> com o jogo pausado. Ver [`ACTIONS_AND_SKILLS.md`](ACTIONS_AND_SKILLS.md).

## 14. Fase

- Tile de `32 × 32 px`.
- Mapa inicial: `100 × 22` tiles (`3200 × 704 px`).
- Elementos: chão contínuo com buracos, plataformas elevadas, blocos sólidos,
  `6` Capangas e uma bandeira no fim.
- Câmera segue o Enzo com suavização e trava nas bordas do mapa.
- Fundo com gradiente e parallax simples.

## 15. Sprites

- Originais, pixel-art, gerados por `scripts/gen-sprites.mjs`.
- Enzo: idle (2 frames), corrida (4), pulo (1), ataque (2).
- Capanga: caminhada (2).
- Tiles: chão, bloco, plataforma.
- Bandeira, projétil Mogging, partículas simples.
- Paleta limitada e consistente; nada copiado de obras existentes.

## 16. Critérios de sucesso do MVP

1. O jogo abre no navegador via build WASM e roda a 60 Hz de simulação.
2. O Enzo anda, corre com double-tap, pula com coyote/buffer, dá dash e colide
   com tiles.
3. `6`/`7` alternados enchem a aura; repetir a tecla zera o combo.
4. Farmar deixa o Enzo lento e vulnerável.
5. O combo de 3 golpes funciona e o terceiro empurra.
6. O Mogging consome aura, acerta inimigos e desaparece em paredes.
7. Capangas patrulham, dão dano de contato e concedem XP ao morrer.
8. Level-up concede ponto, pausa e permite distribuir nos quatro atributos.
9. Morrer respawna mantendo progressão; tocar a bandeira vence.
10. Recarregar a página restaura nível, XP, pontos e atributos do slot carregado.
11. `cargo test -p enzo-core` cobre os cenários do harness e passa.
12. O core não importa render/wasm/IO.
13. O menu inicial permite iniciar, carregar e apagar três slots, e trocar o
    idioma entre pt-BR e inglês.
14. O HUD abre e fecha as quatro janelas pelos dots; elas arrastam, empilham e
    distribuem pontos por clique.

## 17. Premissas a confirmar

1. **Teclas de movimento/pulo:** `A`/`D` ou setas + `Espaço`/`W`.
2. **Mogging atravessa inimigos** (cada um leva dano uma vez) até bater em parede.
3. **Inimigos renascem apenas quando o jogador morre**, não por tempo.
4. **Level-up pausa o jogo** e a distribuição aceita `1`–`4`/clique + `Enter`.
5. **Sem áudio no MVP.**
6. **Valores de balanceamento** da seção 10 são iniciais e ajustáveis.
7. **UI em pt-BR e inglês**, escolhida no menu e persistida localmente.

## 18. Evolução planejada

Nada desta seção está implementado no MVP atual. A ordem abaixo é a ordem de
entrega proposta; cada fase tem plano, contratos e critérios de aceite em
[`ACTIONS_AND_SKILLS.md`](ACTIONS_AND_SKILLS.md).

1. **Input por ação:** teclado vira `InputAction`/`KeyBindings`; o
   `PlayerCommand` passa a ter `move_x`, `move_y` e `slots[4]`.
2. **Skills modulares:** `MoveId`/`MoveSet` viram `SkillId`/`SkillDef`/`Loadout`;
   pulo, dash, voo, soco, Mogging, tiro de nutela e grito ganham módulo próprio,
   requisitos, ícone e estado tipado.
3. **Loadout persistente:** troca de skill no core, save v2, snapshot v3 e menu
   de habilidades com troca/clear e bloqueio por requisito.
4. **Controles mobile:** botões touch emitem `InputAction` diretamente, sem
   passar por teclas físicas.
5. **UI em HTML sobre o canvas:** canvas desenha o mundo; HUD, menus e controles
   mobile viram DOM e conversam com o wasm por uma ponte de ações e `UiState`.
   O HTML não calcula regras nem lê/grava save.

O objetivo de longo prazo é que qualquer skill nova entre como um módulo com
código, requisitos, ícone e arte gerada, sem alterar teclado, HUD ou rede.
