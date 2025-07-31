import {ChangeDetectorRef, Component, ElementRef, OnInit, TemplateRef, ViewChild} from '@angular/core';
import {CommonModule} from '@angular/common';
import {RouterOutlet} from '@angular/router';
import {ShareModule} from "./share.module";
import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";
import {open} from '@tauri-apps/plugin-dialog';
import {NzMessageService} from 'ng-zorro-antd/message';

declare const JSMpeg: any;

@Component({
	selector: 'app-root',
	standalone: true,
	templateUrl: './app.component.html',
	styleUrls: ['./app.component.less'],
	imports: [RouterOutlet, ShareModule],
})
export class AppComponent implements OnInit {
	constructor(
		private change: ChangeDetectorRef,
		private message: NzMessageService
	) {
	}

	ngOnInit() {
		this.jiance();
		listen("notify", e => {
			const payload = e.payload as any;
			if (e.payload && payload.evt_type === "Download") {
				this.process = payload.evt_data;
				if (payload.evt_data === 100) this.loadingShow = false
				this.change.detectChanges();
				this.jiance()
			}
		})
	}

	public process = 0;

	public xiazai() {
		this.loadingShow = true
		this.process = 0;
		invoke("download_ffmpeg").then((r) => {
		}).catch(e => {
		})
	}

	//

	public ffmpegReady: boolean

	public jiance() {
		invoke("check_if_ffmpeg").then(r => {
			this.ffmpegReady = r as boolean;
		})
	}

	public mode = '';

	public mirror() {
		this.mode = 'mirror';
		invoke("start_live_server").then(r => {
			const canvas = document.getElementById("video-canvas");
			this.player = new JSMpeg.Player("ws://localhost:30003", {canvas});
		})
	}

	public mirrorStop() {
		this.player.destroy();
		this.player = null
		this.mode = ''
		invoke("end_live_server").then(r => {
			const canvas = document.getElementById("video-canvas");
			this.player = new JSMpeg.Player("ws://localhost:30003", {canvas});
		})
	}

	//
	// public guanbi() {
	// 	invoke("end_live_server").then(r => {
	// 		console.log(123);
	// 	})
	// }
	public drawer = false;
	public player: any;

	public close() {
		this.drawer = false;
	}

	public loadingShow = false;

	public kmCapture() {
		invoke("start_km_capture").then();
	}

	public kmReceive() {
		invoke("start_km_udp_server").then();
	}


	public async sendFile() {
		try {
			const file = await open({multiple: false, directory: false});
			this.loadingShow = true;
			await invoke('transfer_file', {
				filePath: file
			})
		} catch (e) {
			console.log(e);
		}
	}

	public openFolder() {
		invoke("open_folder").then();
	}

	public receiveFile() {
		invoke("receive_file").then();
	}

	public ip = {
		main: '192.168.0.200',
		sub: '192.168.0.28'
	}

	public setting() {
		if (!this.ip.main || !this.ip.sub) {
			this.message.error("IP未填写");
			return
		}
		invoke("set_ip", {data: this.ip}).then( () => {
			this.message.success("设置成功");
		});
	}
}
