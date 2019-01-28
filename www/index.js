import * as wasm from "../pkg";
import nipplejs from "nipplejs";

const game = new wasm.Game();
console.log(game.get_state());

//const world = new wasm.World();

const CELL_SIZE = 20; // px

const canvas = document.getElementById("game-canvas");
canvas.height = window.innerHeight;
canvas.width = window.innerWidth;

const debugEl = document.getElementById("debug");
function debug(info) {
    debugEl.innerHTML = JSON.stringify(info, 0, 2);
}

var isTouchDevice = "ontouchstart" in document.documentElement;
if (isTouchDevice) {
    // screen joysticks

    const moveJoystick = nipplejs.create({
        zone: document.getElementById("left-joystick-zone"),
        color: "blue"
    });

    moveJoystick.on("end move", (event, data) => {
        if (event.type === "end") {
            //world.set_player_speed(0, 0);
            return;
        }

        if (data.direction) {
            let x = data.instance.frontPosition.x / 50;
            let y = data.instance.frontPosition.y / 50;

            //world.set_player_speed(x, y);
        }
    });

    const fireJoystick = nipplejs.create({
        zone: document.getElementById("right-joystick-zone"),
        color: "red"
    });

    const aimEl = document.getElementById("player-aim");

    fireJoystick.on("start end move", (event, data) => {
        switch (event.type) {
            case "start":
                //world.set_firing(true);
                break;
            case "end":
                //world.set_firing(false);
                break;
            case "move":
                if (data.direction) {
                    const frontPosition = data.instance.frontPosition;

                    const player = world.get_player_pos();

                    const aimPosX = player.x * CELL_SIZE + frontPosition.x;
                    const aimPosY = player.y * CELL_SIZE + frontPosition.y;

                    aimEl.style.left = `${aimPosX}px`;
                    aimEl.style.top = `${aimPosY}px`;
                    player.free();

                    world.set_gan_target(aimPosX / CELL_SIZE, aimPosY / CELL_SIZE);
                }
                break;

            default:
                break;
        }
    });
}

// mouse

canvas.addEventListener("mousemove", event => {
    game.set_shoot_point(event.offsetX / CELL_SIZE, event.offsetY / CELL_SIZE);
});

document.addEventListener("mousedown", event => {
    game.set_shooting(true);
});

document.addEventListener("mouseup", event => {
    game.set_shooting(false);
});

// keyboard

const player_speed = { x: 0, y: 0 };

document.addEventListener("keydown", event => {
    if (event.code == "KeyA") player_speed.x = -1;
    if (event.code == "KeyD") player_speed.x = +1;

    if (event.code == "KeyS") player_speed.y = +1;
    if (event.code == "KeyW") player_speed.y = -1;

    game.set_player_direction(player_speed.x, player_speed.y);
});

document.addEventListener("keyup", event => {
    if (event.code == "KeyA" && player_speed.x == -1) player_speed.x = 0;
    if (event.code == "KeyD" && player_speed.x == +1) player_speed.x = 0;

    if (event.code == "KeyS" && player_speed.y == +1) player_speed.y = 0;
    if (event.code == "KeyW" && player_speed.y == -1) player_speed.y = 0;

    game.set_player_direction(player_speed.x, player_speed.y);
});

const ctx = canvas.getContext("2d");

function draw(state) {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.strokeRect(0, 0, canvas.width, canvas.height);

    // ctx.strokeStyle = "#444";
    // ctx.fillText(world.get_scope(), 950, 50);

    ctx.strokeStyle = "#000";

    for (let i = 0; i < state.enemies.length; ++i) {
        const enemy = state.enemies[i];
        ctx.beginPath();
        ctx.arc(enemy.x * CELL_SIZE, enemy.y * CELL_SIZE, CELL_SIZE * enemy.radius, 0, 2 * Math.PI);
        ctx.stroke();
    }

    const player = state.player;

    ctx.beginPath();
    ctx.arc(player.x * CELL_SIZE, player.y * CELL_SIZE, CELL_SIZE / 4, 0, 2 * Math.PI);
    ctx.stroke();

    // const latest_heat = world.latest_heat();
    // ctx.beginPath();
    // ctx.moveTo(latest_heat.x * CELL_SIZE - CELL_SIZE / 8, latest_heat.y * CELL_SIZE - CELL_SIZE / 8);
    // ctx.lineTo(latest_heat.x * CELL_SIZE + CELL_SIZE / 8, latest_heat.y * CELL_SIZE + CELL_SIZE / 8);
    // ctx.moveTo(latest_heat.x * CELL_SIZE - CELL_SIZE / 8, latest_heat.y * CELL_SIZE + CELL_SIZE / 8);
    // ctx.lineTo(latest_heat.x * CELL_SIZE + CELL_SIZE / 8, latest_heat.y * CELL_SIZE - CELL_SIZE / 8);
    // ctx.stroke();
    // latest_heat.free();
}

setInterval(() => {
    const state = JSON.parse(game.get_state());
    draw(state);
    game.step();
}, 20);
