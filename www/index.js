import * as wasm from "elementals";
import { World } from "elementals";

const world = new World();

const CELL_SIZE = 20; // px
const CANVAS_SIZE = 1000; // px

const canvas = document.getElementById("game-canvas");
canvas.height = CANVAS_SIZE;
canvas.width = CANVAS_SIZE;

canvas.addEventListener("mousemove", event => {
    world.set_gan_target(event.offsetX / CELL_SIZE, event.offsetY / CELL_SIZE);
});

document.addEventListener("mousedown", event => {
    console.log(event);
    world.set_firing(true);
});

document.addEventListener("mouseup", event => {
    world.set_firing(false);
});

let player_speed = { x: 0, y: 0 };

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
    ctx.arc(latest_heat.x * CELL_SIZE, latest_heat.y * CELL_SIZE, CELL_SIZE / 8, 0, 2 * Math.PI);
    ctx.stroke();
    latest_heat.free();
}

setInterval(() => {
    draw();
    world.step();
}, 20);
