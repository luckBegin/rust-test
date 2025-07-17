import {NgModule} from "@angular/core";
import {NzDrawerModule} from 'ng-zorro-antd/drawer';
import {NzTagModule} from 'ng-zorro-antd/tag';
import {NzButtonModule} from 'ng-zorro-antd/button';
import {NzSpinModule} from 'ng-zorro-antd/spin';

const components: any[] = []

const providers: any[] = []

const modules: any[] = [
	NzDrawerModule,
	NzTagModule,
	NzButtonModule,
	NzSpinModule
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
