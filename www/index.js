import * as wasm from "../pkg";
import nipplejs from "nipplejs";

const world = new wasm.World();

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
            world.set_player_speed(0, 0);
            return;
        }

        if (data.direction) {
            let x = data.instance.frontPosition.x / 50;
            let y = data.instance.frontPosition.y / 50;

            world.set_player_speed(x, y);
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
                world.set_firing(true);
                break;
            case "end":
                world.set_firing(false);
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
    world.set_gan_target(event.offsetX / CELL_SIZE, event.offsetY / CELL_SIZE);
});

document.addEventListener("mousedown", event => {
    world.set_firing(true);
});

document.addEventListener("mouseup", event => {
    world.set_firing(false);
});

// keyboard

const player_speed = { x: 0, y: 0 };

document.addEventListener("keydown", event => {
    if (event.code == "KeyA") player_speed.x = -1;
    if (event.code == "KeyD") player_speed.x = +1;

    if (event.code == "KeyS") player_speed.y = +1;
    if (event.code == "KeyW") player_speed.y = -1;

    world.set_player_speed(player_speed.x, player_speed.y);
});

document.addEventListener("keyup", event => {
    if (event.code == "KeyA") player_speed.x = 0;
    if (event.code == "KeyD") player_speed.x = 0;

    if (event.code == "KeyS") player_speed.y = 0;
    if (event.code == "KeyW") player_speed.y = 0;

    world.set_player_speed(player_speed.x, player_speed.y);
});

const ctx = canvas.getContext("2d");

function draw() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.strokeRect(0, 0, canvas.width, canvas.height);

    ctx.strokeStyle = "#444";
    ctx.fillText(world.get_scope(), 950, 50);

    ctx.strokeStyle = "#000";

    for (let i = 0; i < world.enemies_count(); ++i) {
        const enemy = world.enemy(i);
        ctx.beginPath();
        ctx.arc(enemy.pos.x * CELL_SIZE, enemy.pos.y * CELL_SIZE, CELL_SIZE / 2, 0, 2 * Math.PI);
        ctx.stroke();
        enemy.free();
    }

    const player = world.get_player_pos();

    ctx.beginPath();
    ctx.arc(player.x * CELL_SIZE, player.y * CELL_SIZE, CELL_SIZE / 4, 0, 2 * Math.PI);
    ctx.stroke();

    player.free();

    const latest_heat = world.latest_heat();
    ctx.beginPath();
    ctx.moveTo(latest_heat.x * CELL_SIZE - CELL_SIZE / 8, latest_heat.y * CELL_SIZE - CELL_SIZE / 8);
    ctx.lineTo(latest_heat.x * CELL_SIZE + CELL_SIZE / 8, latest_heat.y * CELL_SIZE + CELL_SIZE / 8);
    ctx.moveTo(latest_heat.x * CELL_SIZE - CELL_SIZE / 8, latest_heat.y * CELL_SIZE + CELL_SIZE / 8);
    ctx.lineTo(latest_heat.x * CELL_SIZE + CELL_SIZE / 8, latest_heat.y * CELL_SIZE - CELL_SIZE / 8);
    ctx.stroke();
    latest_heat.free();
}

setInterval(() => {
    draw();
    world.step();
}, 20);
