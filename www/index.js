import * as wasm from "elementals";
import { World } from "elementals";

const world = new World();
//const point = wasm.greet();
console.log(world.enemies_count());
console.log(world.enemy(0));
console.log(world.enemy(0).pos.x);

const CELL_SIZE = 5; // px

const canvas = document.getElementById("game-canvas");
canvas.height = 500;
canvas.width = 500;

const ctx = canvas.getContext("2d");

function draw() {
    ctx.clearRect(0, 0, 500, 500);
    for (let i = 0; i < world.enemies_count(); ++i) {
        const enemy = world.enemy(i);
        ctx.beginPath();
        ctx.arc(enemy.pos.x * CELL_SIZE, enemy.pos.y * CELL_SIZE, CELL_SIZE / 2, 0, 2 * Math.PI);
        ctx.stroke();
        enemy.free();
    }
}

setInterval(() => {
    draw();
    world.step();
}, 20);
