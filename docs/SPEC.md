# EnzoBallZ — Especificação do MVP

> Spec fechado a partir da entrevista `@grillme`. Valores numéricos são o ponto
> de partida; o balanceamento pode mudar sem alterar a arquitetura.

## 1. Objetivo

Um jogo de plataforma 2D estilo Mario, em Rust compilado para WebAssembly, onde
o Enzo corre, pula, luta e farma aura alternando as teclas `6` e `7`. A aura é
mana para o golpe especial **Mogar**. Inimigos dão XP; XP sobe de nível e concede
pontos para distribuir entre Velocidade, Força, Vida e Aura.

## 2. Escopo

- Uma fase com scroll horizontal e tilemap.
- Um jogador no MVP, com core pronto para N jogadores.
- Movimento estilo Mario, incluindo corrida por double-tap.
- Farming de aura com `6`/`7` alternados, combo e lentidão.
- Combo físico de 3 golpes.
- Um golpe especial: **Mogar** (projétil instantâneo).
- Um tipo de inimigo patrulhador.
- XP, level-up, pontos e quatro atributos.
- Vida, aura, morte com respawn e vitória por bandeira.
- Auto-save local da progressão.
- Sprites pixel-art originais gerados por script.

## 3. Não-objetivos do MVP

- Multiplayer em rede, servidor, matchmaking, autenticação.
- Co-op local.
- Mais de uma fase, chefes, itens, loja.
- Outros golpes além do Mogar.
- Áudio.
- Menu inicial, pausa, opções gráficas.
- Mobile/touch.
- Assets de terceiros (incluindo personagens de Dragon Ball ou Mario).

## 4. Loop de jogo

```text
Explorar e enfrentar inimigos
        │
        ├─ ataque físico (J) ──▶ dano grátis, curto alcance
        │
        ├─ farmar aura (6/7) ──▶ enche a barra, mas deixa o Enzo lento
        │
        └─ Mogar (K) ─────────▶ gasta aura, dano à distância
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
| Ataque físico | `J` | combo de 3 |
| Mogar | `K` | custa aura |
| Farmar aura | `6` e `7` | alternância obrigatória |
| Distribuir ponto | `1` `2` `3` `4` | na tela de level-up |
| Confirmar | `Enter` | fecha a tela de level-up |
| Reiniciar fase | `R` | na tela de vitória |

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
- Colisão AABB contra tiles; resolução em X e depois em Y.

## 7. Aura e farming

- Barra de aura base: `100`.
- Alternar `6 → 7 → 6 → 7` conta como acerto.
- Repetir a mesma tecla (ex.: `6, 6`) zera o combo e não gera aura.
- Cada acerto gera `2` de aura × multiplicador.
- Multiplicador: `1 + 0,15 × combo`, limitado a `3,0`.
- Janela do combo: `0,9 s` sem acerto → combo volta a zero.
- Enquanto o combo está ativo, a velocidade do Enzo cai para `45%`.
- A aura não regenera sozinha. Só farming recompõe.
- A aura é gasta apenas no Mogar.

## 8. Combate

### 8.1 Combo físico (`J`)

- Três golpes encadeados dentro de `0,5 s` entre um e outro.
- Dano base: `6`, `6`, `12` (+ bônus de Força).
- Alcance: `34 px` na direção em que o Enzo olha, altura `26 px`.
- O terceiro golpe aplica knockback de `220 px/s`.
- Sem custo de aura.
- Se a janela expirar, o combo reinicia no primeiro golpe.
- O Enzo pode andar durante o ataque.

### 8.2 Mogar (`K`)

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
| Aura | `+10` de aura máxima e `+2` de dano no Mogar |

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

- Chave: `enzoballz.save.v1` no `localStorage`.
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

- Gravado ao subir de nível, ao confirmar a distribuição e ao vencer.
- Posição, inimigos e projéteis não são salvos.
- Versão incompatível → save ignorado com aviso no console (sem crash).

## 13. HUD e UI

- Barra de vida (vermelha), aura (ciano) e XP (amarela).
- Nível atual e pontos disponíveis.
- Indicador de combo de aura (`x2`, `x3`...) e janela restante.
- Dica de controles no início da fase.
- Overlay de level-up com os quatro atributos e teclas `1`–`4`.
- Overlay de morte e de vitória.
- Textos em português (pt-BR).

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
- Bandeira, projétil Mogar, partículas simples.
- Paleta limitada e consistente; nada copiado de obras existentes.

## 16. Critérios de sucesso do MVP

1. O jogo abre no navegador via build WASM e roda a 60 Hz de simulação.
2. O Enzo anda, corre com double-tap, pula com coyote/buffer e colide com tiles.
3. `6`/`7` alternados enchem a aura; repetir a tecla zera o combo.
4. Farmar deixa o Enzo lento e vulnerável.
5. O combo de 3 golpes funciona e o terceiro empurra.
6. O Mogar consome aura, acerta inimigos e desaparece em paredes.
7. Capangas patrulham, dão dano de contato e concedem XP ao morrer.
8. Level-up concede ponto, pausa e permite distribuir nos quatro atributos.
9. Morrer respawna mantendo progressão; tocar a bandeira vence.
10. Recarregar a página restaura nível, XP, pontos e atributos.
11. `cargo test -p enzo-core` cobre os cenários do harness e passa.
12. O core não importa render/wasm/IO.

## 17. Premissas a confirmar

1. **Teclas de movimento/pulo:** `A`/`D` ou setas + `Espaço`/`W`.
2. **Mogar atravessa inimigos** (cada um leva dano uma vez) até bater em parede.
3. **Inimigos renascem apenas quando o jogador morre**, não por tempo.
4. **Level-up pausa o jogo** e a distribuição usa `1`–`4` + `Enter`.
5. **Sem áudio no MVP.**
6. **Valores de balanceamento** da seção 10 são iniciais e ajustáveis.
7. **UI em pt-BR.**
