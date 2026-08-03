

# OmegaOS

Un plano de control terminal para ejecutar una flota de agentes de programación con IA en paralelo, donde cada agente obedece el mismo manual de reglas tipado.

[English](README.md) | [Français](README.fr.md) | [Русский](README.ru.md) | [中文](README.zh.md)

[![CI](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml/badge.svg)](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml) ![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg) ![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)

OmegaOS no es una biblioteca que importa. Se instala en una máquina Linux y obtiene el comando `omega`, una TUI para observar y finalizar sesiones, una capa de orquestación que asigna trabajo a los agentes y un puente de Telegram para el control por teléfono. Las nuevas sesiones usan OpenAI Codex de forma predeterminada. Claude Code, Gemini, Pi, Hermes y GLM siguen siendo opciones explícitas. Cada agente recibe un contexto de política compacto, tipado y con alcance de rol, compilado desde la misma doctrina.

Versión actual: consulte [CHANGELOG.md](CHANGELOG.md) (`omega -V` en una máquina instalada). Lo uso a diario; espere algunos bordes sin pulir.

## Instalación

Un comando en una máquina Linux (macOS funciona en su mayoría):

```
npx omega-os
```

Clona el repositorio y ejecuta el instalador detrás de una pantalla de progreso interactiva estilo lluvia de Matrix (escriba para inyectar glifos, `space` para pulsar; `npx omega-os --plain` para una barra simple). Si prefiere hacerlo manualmente:

```
git clone https://github.com/agentik-os/OmegaOS
cd OmegaOS
./install.sh
```

El instalador descarga binarios precompilados de `rmux` + `omega` para su plataforma cuando se publica una versión (verificados por suma de comprobación) y, de lo contrario, vuelve a compilar desde el código fuente; por lo tanto, un clon fresco siempre reproduce el sistema, solo más rápido cuando existe un binario. Fuerce una compilación desde el código fuente con `OMEGA_FROM_SOURCE=1 ./install.sh`.

## Actualización

```
omega update           # obtener + avanzar rápido + reinstalar
omega update --check   # ¿qué cambiaría? (no toca nada)
```

Actualiza el checkout que encuentra (`$OMEGA_SRC`, el directorio actual, luego
`~/Station/SideBusiness/OmegaOS`, `~/Station/OmegaOS`, `~/OmegaOS` — o pase
`--dir`), recompila desde el código fuente y vuelve a ejecutar el instalador. Su estado en `~/.omega`
se preserva: los secretos, proyectos, configuración de Telegram y `config.toml` nunca
se sobrescriben.

Si tiene cambios locales o commits sin enviar en el checkout, la actualización
**se detiene y le avisa** en lugar de tocar su trabajo: haga commit, stash o push,
y luego vuelva a ejecutarlo.

También se mantiene actualizado: cada instalación verifica a diario a las 03:30 e instala
lo que encuentra, omitiendo cualquier noche en la que su checkout tenga trabajo local, un agente
esté en medio de un turno o el mismo commit haya fallado tres veces. La autoinstalación
significa confiar en el repositorio cada noche, por lo que cambiar esta opción es un solo comando:

```
omega config set auto_update check   # avísame en lugar de instalar
omega config set auto_update off     # no hacer nada en absoluto
```

## Primeros 5 minutos

El stack se instala solo; solo quedan las piezas personales. **`omega guide`
imprime la guía paso a paso completa** (también se guarda en `~/.omega/GETTING-STARTED.md`,
y se muestra al final de la instalación). En resumen:

1. **Conectar Codex** *(obligatorio para el runtime predeterminado)*: ejecute `codex login` y luego verifique con `codex login status`. Claude sigue siendo opcional a través de `claude` y `/login`.
2. **Remoto de Telegram** *(recomendado)* — token desde [@BotFather](https://t.me/BotFather), su id desde [@userinfobot](https://t.me/userinfobot), luego `OMEGA_TG_TOKEN=<TOKEN> omega telegram setup <ID> --user-id <ID>` (la forma de variable de entorno mantiene el token fuera de la lista de procesos). Para un tema por proyecto: grupo + Temas activado + bot como administrador → `/setupgroup` → `/sync`.
3. **Claves de servicio** *(opcional)* — `~/.omega/provisioning/services.env` (Vercel / GitHub / Convex / Stripe / OpenAI-para-voz) alimenta el aprovisionamiento automático de nuevas aplicaciones.
4. **Agregar un proyecto** — `omega` → **[N] New Project**, Telegram → *Import from GitHub*, o simplemente coloque un repositorio en `~/Station/<Category>/`.
5. **Verificar** — `omega doctor`: cada línea `[+]`.

Aquí hay una ejecución real de `omega doctor`:

```
OmegaOS doctor

  [+] binary           omega 0.1.6
  [+] rmux daemon      connected, 6 live session(s)
  [+] rmux socket      /tmp/rmux-1000/default
  [+] doctrine         7 Laws + 47 Rules
  [+] agent CLI        codex available
  [+] state dir        /home/vibe/.omega/state
  [+] telegram service omega-tg-bot active
  [+] hooks            track + verify present, registered in settings.json
  [+] secrets dir      /home/vibe/.omega present
  [+] memory           249088MB available
  [+] usage cache      usage cache 1 min old
  [+] codex auth       Codex login valid
  [+] telegram poller  1 poller
  [+] provisioning     provisioning: VERCEL_TOKEN, CONVEX_TEAM_TOKEN, STRIPE_SECRET_KEY
```

Las líneas `[!]` son advertencias con el comando de reparación en línea; `omega doctor --fix` repara las mecánicas.

## Qué puede hacer

- **Despachar misiones.** `omega dispatch <Project> "<mission>"` asigna trabajo al oráculo de ese proyecto, el cual planifica, genera trabajadores y controla el resultado. `omega orchestrate` ejecuta la tubería completa clasificar → planificar → despachar → monitorear → control en un solo comando.
- **Ejecutar planes tipados.** `/omg-planner` descompone una compilación en un DAG tipado (`.planner/tracker.json`); `omega plan-run` lo ejecuta con aplicación estructural de no-se-puede-saltar (Gate) y prueba independiente de comando de verificación (Guardian).
- **Iniciar aplicaciones completas.** `/omg-new-project` aprovisiona Vercel/Convex/GitHub/Clerk/Stripe desde sus claves, monta el stack y luego ejeciona visión → PRD → plan → compilación.
- **Paralelizar con seguridad.** Los trabajadores reclaman sus archivos con bloqueos consultivos reales (`fs2`), y `omega spawn-worker --worktree` le da a cada trabajador paralelo su propio worktree de git con una fusión limpia al final. Una señal de finalización crea un resultado candidato. Solo un verificador independiente y la puerta de aceptación de la misión pueden cerrarlo.
- **Auditar todo.** Un Arsenal de Calidad de 23 auditorías forenses tipo Gestalt-Popper (`codeaudit`, `secaudit`, `perfaudit`, `a11yaudit`, …) seleccionado automáticamente para lo que cambió, más `/omg-acceptance`: una puerta de aceptación autónoma con navegador que barre cada ruta y repara lo que encuentra.
- **Convocar un consejo.** `/omg-llm-council` plantea una pregunta a cuatro modelos diferentes de Claude en paralelo, hace que se revisionen entre sí de forma anónima y sintetiza un veredicto conservando los disensos: sin claves de API, se ejecuta dentro de su sesión existente.
- **Navegar de forma agentica.** `/omg-browser-use` controla un navegador en la nube para tareas que Playwright no puede expresar con scripts.
- **Realizar también la estrategia de lanzamiento al mercado (go-to-market).** Un pack de marketing integrado (investigación de mercado, posicionamiento, estrategia de contenido, redes sociales, email en frío, creatividad para anuncios, estrategia de lanzamiento) más el par de identidad visual Higgsfield.
- **Recibir informes en su teléfono.** Cada misión termina con un informe PDF con marca en el tema de Telegram del proyecto, y una tarjeta de progreso en vivo se actualiza en su lugar mientras se ejecuta. Un bot de depósito les da a los agentes una bandeja de entrada privada para los archivos que envía desde su teléfono.
- **Operarlo.** `omega doctor` (salud del stack completo), `patrol` (supervisor de sesión), `usage` (presupuesto de tokens + alertas de Telegram), `backup` (estado irreproducible `~/.omega` → un tgz), `cleanup` / `kill-all`, `timeline` (reproducir una misión), `resurrect` (revivir un oráculo caído), `provision` (grupos de credenciales por cliente).
- **Resolver tickets de Linear de extremo a extremo.** `/omg-linear` repara, captura evidencia, audita hasta 100/100, comenta y mueve el ticket a revisión: nunca a Done; eso lo hace un humano. Consulte [Integración con Linear](#linear-integration).

Tres formas de acceso: la TUI `ratatui` (5 pestañas: Sesiones, Menú, Agéntico, Configuración, Ayuda), la CLI `omega` (más de 40 comandos) y el centro de Telegram. Un modo RPC (JSONL sobre stdin/stdout) lo impulsa desde otras herramientas. Por debajo, todo se ejecuta en [rmux](https://github.com/agentik-os/rmux), un multiplexor de terminal de Rust: sin dependencia de tmux.

## La doctrina

Existe un registro tipado de 7 Leyes y 47 Reglas operacionales nombradas. `omega rules list` imprime el conjunto actual. El compilador reside en `crates/omega-core/src/rules.rs`; emite un contexto determinista y consciente del proveedor con un presupuesto fijo de 24 KB de OmegaOS.

**Las Leyes son inviolables.** Vinculan a cada agente y anulan cada regla y tarea. Hay siete:

- **L0 — Lanzar la verdad.** Un cambio no está listo hasta que una reconstrucción limpia lo reproduce y se envía (push). Menos que eso es un borrador.
- **L1 — El runtime es la única verdad.** El código y los comentarios declaran la intención. Solo ejecutarlo revela la realidad. Cuando discrepan, el runtime gana.
- **L2 — Investigador, no adulador.** Desafíe una premisa defectuosa con razonamiento antes de actuar. Sin confianza falsa. "Esto debería funcionar" sin evidencia es una mentira.
- **L3 — Decidir y proceder.** Un agente despachado es autónomo. Nunca se detiene a preguntar "¿debería continuar?". Decide, ejecuta e informa después.
- **L4 — Listo significa 100%, verificado.** 92% no es listo. Enumere las tareas, termine cada una, verifique cada una contra el runtime.
- **L5 — Calidad sobre velocidad.** Sin variantes simplistas, ligeras o rápidas de un protocolo real. Un 403 o un 401 es un aborto, no un paso.
- **L6 — Terminar la misión.** Enumere, ejecute, verifique e informe cada entregable solicitado. Un plan o una fase parcial no son puntos de parada válidos.

**Las Reglas son operacionales.** Nombradas (R-SCOPE, R-VERIFY, R-CITE, …) y clasificadas en Universal, QualityGate, Orchestration, Reporting y Safety. Cada regla tiene alcance para los roles que vincula: Maestro, Oráculo, Trabajador. Un trabajador no se carga con reglas de orquestación sobre las que no puede actuar, y un oráculo no carga la disciplina de bloqueo de archivos del trabajador. Mismo registro, diferentes fragmentos.

### El embudo

Este es el mecanismo. `rules::compile_rule_context_for_provider` combina el núcleo compacto de leyes, contrato de rol, reglas relevantes para la misión, mecánicas del proveedor y referencias de habilidades. Rechaza salidas por encima del presupuesto de contexto en lugar de truncar en silencio. Cada contexto compilado tiene un digest determinista para la detección de desviación (drift).

Un trabajador tres niveles abajo en el árbol lleva las mismas siete Leyes que el Maestro. Los procedimientos operacionales se cargan solo cuando el rol, la misión, el riesgo y el proveedor lo requieren. Esto mantiene los invariantes universales sin inyectar cada manual de operaciones en cada turno.

Consulte el conjunto completo:

```
omega rules list
```

![omega rules list: Las Leyes y Reglas, impresas por OmegaOS](assets/omega-rules.svg)

## Arquitectura

Cuatro niveles, de arriba a abajo:

```
┌─────────────────────────────────────────────────────────────────┐
│  Nivel 1 — Interfaz Humana                                      │
│  TUI (5 pestañas) · CLI (40+ cmds) · Centro de Telegram         │
│                      ↓ intención                                │
├─────────────────────────────────────────────────────────────────┤
│  Nivel 2 — Maestro (cerebro persistente: el tema Atlas)         │
│  14 plantillas de agente con nombres de Matrix, clasificar → enrutar│
│                      ↓ despachar                                │
├─────────────────────────────────────────────────────────────────┤
│  Nivel 3 — Oráculo (1 por proyecto)                             │
│  Clasificar → Planificar → Despachar trabajadores → Puerta de calidad│
│                      ↓ descomponer                              │
├─────────────────────────────────────────────────────────────────┤
│  Nivel 4 — Trabajadores (efímeros, paralelos, alcance de bloqueo de archivos)│
│  Ejecutar → Verificar → done.json → Ack del Oráculo → Cerrar     │
└─────────────────────────────────────────────────────────────────┘
```

**Nivel 2: el Maestro.** Un agente persistente que permanece en ejecución, se reinicia automáticamente si muere y reanuda su propia conversación. Incluye 14 plantillas de agente con nombres de personajes de Matrix (Oráculo, Morpheus, Seraph, Keymaker, Smith, Niobe, Architect, Merovingian, Neo, Zion, Link, Construct, Pythia, Council). El Maestro es un despachador. Solo clasifica y enruta el trabajo a los oráculos.

**Nivel 3: Oráculo.** Uno por proyecto. Clasifica la solicitud, planifica, despacha trabajadores y ejecuta la puerta de calidad al final. Un oráculo orquesta. No edita el código del proyecto directamente, por lo que el evaluador y el escritor nunca son el mismo agente.

**Nivel 4: Trabajadores.** Efímeros. Se ejecutan en paralelo, cada uno con alcance a sus propios archivos mediante un reclamo de bloqueo de archivos (bloqueos consultivos a través de `fs2`) — y opcionalmente a su propio worktree de git. Un trabajador señala finalización escribiendo un `done.json` con estado `done_clean`, `pending` o `failed`; sin ese estado no está listo.

### Cómo se ejecuta una misión

Una solicitud entra por la TUI, la CLI o Telegram. Dondequiera que comience, aterriza en el Maestro, que la lee, la clasifica y la enruta al oráculo que posee el proyecto relevante. El oráculo planifica la misión, la divide en tareas y despacha un trabajador por tarea. Los trabajadores verifican sus propios resultados contra el runtime real y escriben su `done.json`; el oráculo lo lee, ejecuta la puerta e informa a lo largo de la cadena.

Un trabajador no tiene que procesar sus subtareas una por una. Puede ejecutar un flujo de trabajo en proceso: generar subagentes en paralelo, verificar sus salidas y combinarlas en una sola respuesta. La revisión de código usa esto, al igual que la investigación, las auditorías y el trabajo de diseño.

La verificación es deliberadamente adversarial: un trabajador que informa "listo" no termina la comprobación; su reclamo va a agentes independientes y solo sobrevive si una mayoría (dos de tres) está de acuerdo. Las auditorías del Arsenal de Calidad se conectan justo aquí, en la puerta.

Esto depende del embudo de doctrina anterior: cada agente, en cada nivel, recibe sus Leyes y Reglas con alcance de rol inyectadas en el momento en que se despachan.

Esta sección del README es en sí un ejemplo. Un flujo de trabajo lo produjo. Un agente escribió el borrador, lectores independientes lo revisaron buscando prosa generada por IA, otro agente lo revisó contra lo que marcaron y hablantes nativos manejaron la traducción. Así que ninguna parte de este texto proviene de un solo paso sin revisar.

## Stack

Es un workspace de Rust con tres crates:

- `omega-core` — orquestación, el registro de reglas, doctor, timeline, cleanup, patrol, bloqueo de alcance de archivos.
- `omega-cli` — el binario `omega`, construido sobre `clap`.
- `omega-tui` — el gestor de sesiones, construido sobre `ratatui`.

Por debajo, se ejecuta en [rmux](https://github.com/agentik-os/rmux), un multiplexor de terminal de Rust: un daemon, un SDK tipado y manejo de PTY. rmux es una biblioteca tipada de Rust, por lo que OmegaOS la llama directamente en lugar de ejecutar tmux y analizar texto. No hay ninguna dependencia de tmux en ningún lado.

Bun y TypeScript se encargan del renderizado de informes PDF (a través de Next.js y Playwright) y de los bots de Telegram. Bash aparece en exactamente un lugar: el bootstrap de instalación.

## Conexión remota

El daemon rmux posee cada sesión, por lo que sus agentes continúan ejecutándose después de que se desconecte. Para volver a ellos, **adjunte** — reconecte su terminal a una sesión que ya esté en ejecución:

```
rmux attach              # re-adjuntar a la última sesión
rmux attach -t claude-1  # adjuntar a una específica
rmux list-sessions       # ver lo que está en vivo
```

Desconecte nuevamente con `Ctrl-b d` — la sesión y sus agentes continúan ejecutándose sin usted.

`omega` envuelve los puntos de entrada que realmente usa:

```
omega                       # abrir el gestor de sesiones TUI (navegar / lanzar / monitorear)
omega attach -t claude-1    # entrar directamente a una sesión para trabajar en ella
omega master                # saltar a la sesión del Maestro
omega list                  # listar cada sesión en vivo
```

Use el menú (`omega`) para gestionar y lanzar; use un adjuntar directo (`omega attach -t …`, o `rmux attach -t …`) cuando quiera escribir concentrado en una sola sesión: la vista previa del menú *espeja* el panel, mientras que un adjuntar directo es la ruta de menor latencia.

Por SSH desde una laptop, el SSH plano espera una ida y vuelta completa de red antes de reflejar cada tecla, por lo que en una máquina remota escribir se siente con retraso y la salida del agente llega en fragmentos: sin importar qué tan rápida sea la máquina, porque es latencia, no CPU. `install.sh` instala [`mosh`](https://mosh.org) para esto: refleja sus teclas localmente y envía diferencias de pantalla por UDP, por lo que escribir es instantáneo y la transmisión es suave a cualquier latencia. Conéctese directamente a una sesión con:

```
mosh user@host -- omega attach -t claude-1
```

En un cliente como **Termius**: configure la IP + puerto del host, active la opción **mosh** y agregue un fragmento de inicio: `omega` para el menú, o `omega attach -t <session>` para aterrizar directamente en una sesión.

(Use `Alt+Arriba/Abajo` de rmux para el desplazamiento, no PageUp de mosh.) El instalador también configura `/etc/rmux.conf` y un locale UTF-8 a nivel del sistema, por lo que cada cuenta — root y futuros usuarios — obtiene la misma sesión endurecida (desplazamiento con mouse, selección con arrastrar al portapapeles local por SSH, teclas rápidas, truecolor) sin configuración por usuario.

## Integración con Linear

Si rastrea comentarios de usuarios en [Linear](https://linear.app), OmegaOS resuelve los tickets de extremo a extremo. Dos comandos.

`/omg-linear-setup` es un asistente único, ejecutado dentro de su propia aplicación. Instala un widget de comentarios en la aplicación (captura una captura de pantalla, la URL de la página, el elemento clickeado y la consola del navegador en el momento del informe), las etiquetas de Linear en las que la tubería hace pivot, y la ruta de API que convierte un informe del widget en un issue de Linear. Detecta su stack, proveedor de auth y biblioteca de UI primero, por lo que escribe código que encaja con el proyecto en lugar de una plantilla genérica.

`/omg-linear` hace el trabajo. Lee los tickets abiertos y, para cada uno, repara el código, captura evidencia antes/después y luego ejecuta las auditorías del Arsenal de Calidad que encajan con el cambio. Un ticket solo avanza si esas auditorías llegan a 100/100. Luego publica un comentario de verificación de reparación en el ticket y lo mueve a un estado de revisión: `In Review` si su equipo tiene uno, de lo contrario un `Omega Review` neutral que crea. Nunca marca un ticket como Done; un humano lo hace después de verificarlo. El motor v2 ejecuta esto a través de un Workflow: clasifica los tickets abiertos, distribuye la reparación y auditoría por ticket en paralelo y verifica cada resolución de forma adversarial antes de comentar.

Tiene guarda-desencadenadores. OmegaOS solo toca Linear cuando se lo pide por nombre (`/omg-linear`, `fix linear`, un id de ticket como `KOM-7`, o un enlace de `linear.app`). La palabra suelta "feedback" nunca lo activa, y no mencionará Linear a menos que usted lo haga.

```
omega_dir=~/.omega          # el protocolo se envía a ~/.omega/skills/linear/
/omg-linear-setup           # una vez por app: instala el widget + etiquetas + ruta
/omg-linear                 # resolver tickets abiertos: reparar -> auditar -> comentar -> In Review
```

## Limitaciones

Prefiero que lo sepa de antemano.

- **Primero Linux.** Desarrollado en un VPS sin cabeza. No Windows. macOS recibe correcciones reales (servicios launchd, ruta Homebrew) pero está menos probado.
- La TUI asume un terminal de 256 colores. En un terminal de 16 colores se verá feo.
- El runtime de agente predeterminado es OpenAI Codex, por lo que la CLI `codex` debe estar iniciada sesión. Claude Code, Gemini, Pi, Hermes y GLM son alternativas explícitas soportadas.
- **Máquina única.** El daemon rmux es local. No hay orquestación multihost.
- Es 0.1.x. Lo uso a diario, pero encontrará bordes sin pulir que aún no he golpeado.

## Lea GUIDE.md a continuación

**[GUIDE.md](GUIDE.md)** es el manual del operador: el vocabulario (misión, oráculo, trabajador, objetivo, plan, Atlas), las tres cabinas de mando, sus primeras misiones, el catálogo de habilidades y cómo se verifica el trabajo. Luego profundice:

- [docs/FEATURES.md](docs/FEATURES.md) — **el catálogo completo de características** (cada subsistema + cómo acceder a él).
- [docs/README.md](docs/README.md) — el índice de documentación.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — la referencia del sistema completo.
- [docs/MAP.md](docs/MAP.md) — dónde reside todo en disco.
- [docs/THEMES.md](docs/THEMES.md) — la galería de paletas de la TUI.
- [docs/RESET-RECOVERY.md](docs/RESET-RECOVERY.md) — respaldar y reconstruir una máquina.
- [CHANGELOG.md](CHANGELOG.md) — qué se lanzó, versión por versión.

## Créditos

OmegaOS se construye sobre mucho trabajo de otras personas:

La mayor deuda es con [rmux](https://github.com/agentik-os/rmux), el multiplexor de terminal de Rust sobre el que se ejecuta todo aquí.

El resto del stack de Rust:

- [ratatui](https://github.com/ratatui/ratatui) y [crossterm](https://github.com/crossterm-rs/crossterm) — la TUI.
- [tokio](https://github.com/tokio-rs/tokio) — el runtime asíncrono.
- [clap](https://github.com/clap-rs/clap) y `clap_complete` — la CLI y autocompletados de shell.
- [serde](https://github.com/serde-rs/serde) con `serde_json`, `serde_yaml` y `toml` — configuración y estado.
- [anyhow](https://github.com/dtolnay/anyhow) y [thiserror](https://github.com/dtolnay/thiserror) — manejo de errores.
- `chrono` (marcas de tiempo), `dirs` (rutas), `fs2` (los bloqueos consultivos de archivos detrás de los reclamos de alcance), `regex`, `tempfile`, `tracing` con `tracing-subscriber` (registro) y `reqwest` (HTTP de Telegram y PDF).

[Claude Code](https://www.anthropic.com) de Anthropic es el runtime de agente.

## Licencia

Doble licencia bajo cualquiera de [MIT](LICENSE-MIT) o [Apache-2.0](LICENSE-APACHE), a su elección. Convención estándar de Rust. Elija la que prefiera.
