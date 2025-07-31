import {NgModule} from "@angular/core";
import {NzDrawerModule} from 'ng-zorro-antd/drawer';
import {NzTagModule} from 'ng-zorro-antd/tag';
import {NzButtonModule} from 'ng-zorro-antd/button';
import {NzSpinModule} from 'ng-zorro-antd/spin';
import { FormsModule } from '@angular/forms';
import { NzInputModule } from 'ng-zorro-antd/input';
import {BrowserAnimationsModule, NoopAnimationsModule} from "@angular/platform-browser/animations";
import {CommonModule} from "@angular/common";
const components: any[] = []

const providers: any[] = []

const modules: any[] = [
	NzDrawerModule,
	NzTagModule,
	NzButtonModule,
	NzSpinModule,
	FormsModule,
	NzInputModule,
	CommonModule,
]

@NgModule({
	declarations: [
		...components,
	],
	providers: [
		...providers
	],
	imports: [
		...modules
	],
	exports: [
		...modules
	]
})
export class ShareModule {

}
