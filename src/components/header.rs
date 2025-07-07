use leptos::*;
use leptos_router::*;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="bg-white shadow-sm border-b">
            <div class="container mx-auto px-4">
                <div class="flex items-center justify-between h-16">
                    <div class="flex items-center space-x-4">
                        <A href="/" class="text-xl font-bold text-gray-800 hover:text-blue-600">
                            "URL翻译工具"
                        </A>
                    </div>
                    
                    <nav class="flex items-center space-x-6">
                        <A 
                            href="/" 
                            class="text-gray-600 hover:text-blue-600 transition-colors"
                            active_class="text-blue-600 font-medium"
                        >
                            "首页"
                        </A>
                        <A 
                            href="/settings" 
                            class="text-gray-600 hover:text-blue-600 transition-colors"
                            active_class="text-blue-600 font-medium"
                        >
                            "设置"
                        </A>
                    </nav>
                </div>
            </div>
        </header>
    }
}