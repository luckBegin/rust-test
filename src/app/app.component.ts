import {Component, OnInit} from '@angular/core';
import {CommonModule} from '@angular/common';
import {RouterOutlet} from '@angular/router';
import {invoke} from "@tauri-apps/api/core";
import {listen} from '@tauri-apps/api/event'

@Component({
	selector: 'app-root',
	standalone: true,
	imports: [CommonModule, RouterOutlet],
	templateUrl: './app.component.html',
	styleUrl: './app.component.css'
})
export class AppComponent implements OnInit {
	ngOnInit() {
		listen("aaaa", e => {
			console.log(123, e);
		})
	}

	public click() {
		invoke("devices").then((r) => {
			console.log(r);
		})
	}
}
