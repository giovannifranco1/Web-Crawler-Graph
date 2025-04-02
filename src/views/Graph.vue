<script setup>
import { onMounted, readonly, ref } from "vue";
import * as d3 from "d3";

// Adicionar uma função de throttle para limitar a frequência de atualizações
function throttle(func, limit) {
    let inThrottle;
    return function() {
        const args = arguments;
        const context = this;
        if (!inThrottle) {
            func.apply(context, args);
            inThrottle = true;
            setTimeout(() => inThrottle = false, limit);
        }
    };
}

// Função de debounce para evitar chamadas repetidas
function debounce(func, delay) {
    let debounceTimer;
    return function() {
        const context = this;
        const args = arguments;
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => func.apply(context, args), delay);
    };
}

const globalUrls = readonly(
    JSON.parse(localStorage.getItem("global-urls")) ?? []
);

// Armazena os IDs dos nós principais que foram clicados (expandidos)
const expandedNodes = ref(new Set());
// Armazena os IDs dos nós principais que estão com o mouse em cima (hover)
const hoveredNodes = ref(new Set());
// Detecta se estamos em um dispositivo touch
const isTouchDevice = ref(false);
// Detecta se estamos em um dispositivo de baixo desempenho
const isLowPerfDevice = ref(false);
// Permite ao usuário ativar manualmente o modo de baixo desempenho
const manualLowPerfMode = ref(false);
// Flag combinada para determinar se o modo de baixo desempenho está ativo
const lowPerfModeActive = ref(false);
// Flag para controlar se a animação de crescimento está ativada
const animationEnabled = ref(localStorage.getItem('graph-animation-enabled') !== 'false');
// Para rastrear o último nó clicado e o tempo para detectar duplo clique
const lastClickedNode = ref(null);
const lastClickTime = ref(0);
// Armazena os IDs dos nós secundários que foram clicados para mostrar nome
const clickedSubNodes = ref(new Set());
// Referência ao SVG e elementos do grafo para reiniciar a animação
const svgRef = ref(null);
const nodeRef = ref(null);
const linkRef = ref(null);
const textRef = ref(null);

// Variáveis para armazenar referências aos elementos globalmente
let activeNodes = null;
let activeLinks = null;
let activeTexts = null;
let activeSimulation = null;
let activeWidth = 0;
let activeHeight = 0;
let activeParentMap = null;
let activeChildrenMap = null;
let activeTicked = null;

onMounted(() => {
    // Verificar se estamos em um dispositivo touch
    isTouchDevice.value = ('ontouchstart' in window) || 
                          (navigator.maxTouchPoints > 0) || 
                          (navigator.msMaxTouchPoints > 0);
                          
    // Verificar se estamos em um dispositivo de baixo desempenho
    // Consideramos dispositivos móveis ou com muitos nós como baixo desempenho
    isLowPerfDevice.value = isTouchDevice.value || window.innerWidth < 768 || 
                            (globalUrls.sub_urls && globalUrls.sub_urls.length > 15);
                            
    // Definir se o modo de baixo desempenho está ativo
    lowPerfModeActive.value = isLowPerfDevice.value || manualLowPerfMode.value;
    
    console.log("globalUrls from localStorage:", localStorage.getItem("global-urls"));
    console.log("parsed globalUrls:", globalUrls);
    
    if (globalUrls.length === 0) {
        console.log("No URLs found in localStorage");
        return;
    }

    drawGraph();
    
    // Adicionar listener para redimensionamento com debounce
    window.addEventListener('resize', debounceResize);
});

// Função para lidar com redimensionamento da janela com debounce
const debounceResize = debounce(() => {
    handleResize();
}, 250); // Espera 250ms após o último evento de redimensionamento

function handleResize() {
    // Redesenha o grafo ao redimensionar a janela
    const graphContainer = document.querySelector('.graph-container');
    if (graphContainer) {
        const legendElement = document.querySelector('.legend');
        const indicatorElement = document.querySelector('.touch-indicator');
        const performanceToggleElement = document.querySelector('.performance-toggle');
        const animationToggleElement = document.querySelector('.animation-toggle');
        const restartButtonElement = document.querySelector('.restart-button');
        
        // Preservar os elementos de UI existentes
        graphContainer.innerHTML = '<svg id="graph"></svg>';
        if (legendElement) graphContainer.appendChild(legendElement);
        if (indicatorElement) graphContainer.appendChild(indicatorElement);
        if (performanceToggleElement) graphContainer.appendChild(performanceToggleElement);
        if (animationToggleElement) graphContainer.appendChild(animationToggleElement);
        if (restartButtonElement) graphContainer.appendChild(restartButtonElement);
        
        drawGraph();
    }
}

function prepareGraphData(urls) {
    console.log("prepareGraphData received urls:", urls);
    let nodes = [];
    let links = [];
    let parentMap = new Map(); // Mapa para rastrear o pai de cada nó
    let childrenMap = new Map(); // Mapa para rastrear os filhos de cada nó
    
    function traverse(node, parent = null, isRoot = false) {
        console.log("traversing node:", node);
        // Uma URL é principal se tiver sub_urls ou for a raiz
        const hasSubUrls = node.sub_urls && node.sub_urls.length > 0;
        
        nodes.push({ 
            id: node.url,
            isMainUrl: hasSubUrls || isRoot // URL é principal se tiver sub_urls ou for a raiz
        });

        if (parent) {
            links.push({ source: parent.url, target: node.url });
            parentMap.set(node.url, parent.url); // Armazenar o relacionamento pai-filho
            
            // Adicionar este nó como filho do pai
            if (!childrenMap.has(parent.url)) {
                childrenMap.set(parent.url, []);
            }
            childrenMap.get(parent.url).push(node.url);
        }

        // Passamos false para isRoot nos nós filhos
        node.sub_urls.forEach((sub) => traverse(sub, node, false));
    }

    // Começamos com isRoot = true para a raiz
    traverse(urls, null, true);

    return { nodes, links, parentMap, childrenMap };
}

function drawGraph() {
    const { nodes, links, parentMap, childrenMap } = prepareGraphData(globalUrls);
    const width = window.innerWidth - 40; // Mais largo para ocupar a tela
    const height = window.innerHeight - 40; // Mais alto para ocupar a tela

    // Atualizar as variáveis globais com as referências ativas
    activeWidth = width;
    activeHeight = height;
    activeParentMap = parentMap;
    activeChildrenMap = childrenMap;

    const svg = d3.select("#graph")
        .attr("width", width)
        .attr("height", height);

    // Adicionar um grupo para os elementos do grafo
    const g = svg.append("g");
    
    // Adicionar um retângulo de fundo para capturar eventos
    g.append("rect")
        .attr("width", width * 3) // Área maior para permitir pan
        .attr("height", height * 3)
        .attr("x", -width)
        .attr("y", -height)
        .attr("fill", "none")
        .attr("pointer-events", "all");
    
    // Criar grupo para links e nós
    const linksGroup = g.append("g").attr("class", "links");
    const nodesGroup = g.append("g").attr("class", "nodes");
    const textsGroup = g.append("g").attr("class", "texts");
    
    // Flag para controlar o modo de renderização
    let isZooming = false;
    let currentZoomScale = 1;
    
    // Configurar zoom com throttling
    const zoom = d3.zoom()
        .scaleExtent([0.1, 4]) // Define zoom limits
        .on("start", () => {
            isZooming = true;
            // Em dispositivos de baixo desempenho, ocultar completamente os textos durante zoom
            if (lowPerfModeActive.value) {
                textsGroup.style("display", "none");
            } else {
                // Apenas reduzir a opacidade em dispositivos de alto desempenho
                textsGroup.style("opacity", 0.3);
            }
        })
        .on("zoom", throttle((event) => {
            g.attr("transform", event.transform);
            currentZoomScale = event.transform.k;
            
            // Ajustar a aparência com base no nível de zoom
            updateVisualsBasedOnZoom(currentZoomScale);
        }, lowPerfModeActive.value ? 30 : 10)) // Throttle mais agressivo em dispositivos de baixo desempenho
        .on("end", () => {
            isZooming = false;
            // Restaurar visualização do texto após zoom
            if (lowPerfModeActive.value) {
                textsGroup.style("display", "");
            }
            textsGroup.style("opacity", 1);
            
            // Ajustar a aparência final com base no nível de zoom
            updateVisualsBasedOnZoom(currentZoomScale);
        });

    // Função para ajustar a aparência com base no nível de zoom
    function updateVisualsBasedOnZoom(scale) {
        // Quando o zoom está muito longe, simplificar a visualização
        if (scale < 0.4) {
            // Reduzir tamanhos dos elementos
            node.attr("r", d => {
                if (d.isMainUrl) return Math.max(4, 12 * scale);
                return Math.max(2, 6 * scale);
            });
            
            // Ocultar textos quando muito longe
            if (scale < 0.25) {
                textsGroup.style("display", "none");
            } else {
                textsGroup.style("display", "");
                text.attr("font-size", `${Math.max(8, 12 * scale)}px`);
            }
            
            // Reduzir espessura das linhas
            link.attr("stroke-width", Math.max(0.5, 1.5 * scale));
        } else {
            // Restaurar aparência normal
            node.attr("r", d => d.isMainUrl ? 12 : 6);
            textsGroup.style("display", "");
            text.attr("font-size", "12px");
            link.attr("stroke-width", 1.5);
        }
    }

    svg.call(zoom);

    // Otimizar a simulação de força com configurações mais eficientes
    const simulation = d3
        .forceSimulation(nodes)
        .force(
            "link",
            d3
                .forceLink(links)
                .id((d) => d.id)
                .distance(lowPerfModeActive.value ? 100 : 150) // Menor distância em dispositivos de baixo desempenho
        )
        .force("charge", d3.forceManyBody()
            .strength(lowPerfModeActive.value ? -200 : -300)
            .distanceMax(lowPerfModeActive.value ? 300 : 500)) // Parâmetros mais leves para dispositivos de baixo desempenho
        .force("center", d3.forceCenter(width / 2, height / 2))
        .alphaDecay(lowPerfModeActive.value ? 0.1 : 0.05) // Simulação converge mais rápido em dispositivos de baixo desempenho
        .velocityDecay(lowPerfModeActive.value ? 0.4 : 0.3);
        
    // Salvar a referência da simulação para acesso posterior
    svg.property("__simulation__", simulation);

    // Criar elementos gráficos
    const link = linksGroup
        .selectAll("line")
        .data(links)
        .enter()
        .append("line")
        .attr("stroke", "#808080")
        .attr("stroke-width", 1.5)
        .style("opacity", 0); // Começar invisível para animação

    const node = nodesGroup
        .selectAll("circle")
        .data(nodes)
        .enter()
        .append("circle")
        .attr("r", d => d.isMainUrl ? 12 : 6)
        .attr("fill", d => getNodeFill(d))
        .attr("stroke", d => {
            if (expandedNodes.value.has(d.id)) return "#FFFFFF"; // Expandido tem borda branca
            if (clickedSubNodes.value.has(d.id)) return "#00CCFF"; // Sub_url clicada tem borda azul
            return "none"; // Outros não têm borda
        })
        .attr("stroke-width", 2)
        .style("cursor", "pointer") // Todos os nós são clicáveis agora
        .attr("tabindex", d => d.isMainUrl ? 0 : -1) // Apenas URLs principais são focáveis pelo teclado
        .attr("role", d => d.isMainUrl ? "button" : "presentation") // Papel semântico para acessibilidade
        .attr("aria-label", d => d.isMainUrl ? `URL: ${d.id}` : "") // Descrição para leitores de tela
        .style("opacity", 0) // Começar invisível para animação
        .call(drag(simulation))
        .on("click", handleNodeClick) // Agora lida com clique em todos os nós
        .on("focus", handleNodeFocus)
        .on("blur", handleNodeBlur)
        .on("keydown", handleKeyDown); // Expandir/colapsar com teclado

    const text = textsGroup
        .selectAll("text")
        .data(nodes)
        .enter()
        .append("text")
        .text((d) => getNodeText(d))
        .attr("font-size", "12px")
        .attr("dx", 10)
        .attr("dy", 4)
        .attr("fill", "#fff")
        .style("pointer-events", "none") // Evitar que textos capturem eventos de mouse
        .style("opacity", 0); // Começar invisível para animação

    // Função para determinar se o texto deve ser exibido
    function getNodeText(d) {
        if (d.isMainUrl) return d.id; // URLs principais sempre mostram texto
        
        // Para sub_urls, verifique se o pai está expandido, com hover, ou se foi clicada diretamente
        const parentId = parentMap.get(d.id);
        if (clickedSubNodes.value.has(d.id) || 
            (parentId && (expandedNodes.value.has(parentId) || hoveredNodes.value.has(parentId)))) {
            return d.id; // Mostra o texto se foi clicada ou se o pai estiver expandido/hover
        }
        
        return ""; // Caso contrário, não mostra texto
    }

    // Função para determinar a cor de preenchimento do nó
    function getNodeFill(d) {
        if (d.isMainUrl) return "#FF4444"; // URLs principais são vermelhas
        
        // Verificar se este nó é filho de um nó expandido ou com hover
        const parentId = parentMap.get(d.id);
        if (parentId && (expandedNodes.value.has(parentId) || hoveredNodes.value.has(parentId))) {
            return "#AADDFF"; // Filhos de nós expandidos/hover são azuis claros
        }
        
        return "#888888"; // Outros nós são cinzas
    }

    // Manipulador para clique nos nós
    function handleNodeClick(event, d) {
        const currentTime = new Date().getTime();
        const timeDiff = currentTime - lastClickTime.value;

        if (d.isMainUrl) {
            // Lógica para URLs principais - detecção de duplo clique
            if (lastClickedNode.value === d.id && timeDiff < 300) {
                // Duplo clique - expandir/colapsar
                toggleNodeExpansion(d);
                lastClickedNode.value = null; // Resetar para não detectar triplo clique
            } else {
                // Clique único - apenas dar foco
                handleNodeFocus(event, d);
                lastClickedNode.value = d.id;
                lastClickTime.value = currentTime;
            }
        } else {
            // Lógica para sub_urls - alternar visibilidade do nome
            if (clickedSubNodes.value.has(d.id)) {
                clickedSubNodes.value.delete(d.id); // Ocultar nome
            } else {
                clickedSubNodes.value.add(d.id); // Mostrar nome
            }
            
            // Atualizar textos
            text.text(getNodeText);
            
            // Atualizar bordas para mostrar visualmente quais sub_urls estão selecionadas
            node.attr("stroke", n => {
                if (expandedNodes.value.has(n.id)) return "#FFFFFF"; // Expandido tem borda branca
                if (hoveredNodes.value.has(n.id)) return "#AAAAAA"; // Hover tem borda cinza
                if (clickedSubNodes.value.has(n.id)) return "#00CCFF"; // Sub_url clicada tem borda azul
                return "none"; // Outros não têm borda
            });
        }
    }
    
    // Função para expandir/colapsar um nó
    function toggleNodeExpansion(d) {
        if (expandedNodes.value.has(d.id)) {
            expandedNodes.value.delete(d.id); // Colapsar
        } else {
            expandedNodes.value.add(d.id); // Expandir
        }
        
        // Atualizar os textos
        text.text(getNodeText);
        
        // Atualizar a borda dos nós para mostrar quais estão expandidos
        node.attr("stroke", n => {
            if (expandedNodes.value.has(n.id)) return "#FFFFFF"; // Expandido tem borda branca
            if (hoveredNodes.value.has(n.id)) return "#AAAAAA"; // Hover tem borda cinza
            if (clickedSubNodes.value.has(n.id)) return "#00CCFF"; // Sub_url clicada tem borda azul
            return "none"; // Outros não têm borda
        });
        
        // Atualizar o padrão de traço para as bordas
        node.attr("stroke-dasharray", n => {
            if (hoveredNodes.value.has(n.id) && !expandedNodes.value.has(n.id)) return "3,3";
            return "0";
        });
        
        // Atualizar as cores dos nós
        node.attr("fill", getNodeFill);
    }

    // Manipulador para quando o nó recebe focus
    function handleNodeFocus(event, d) {
        if (!d.isMainUrl) return; // Apenas URLs principais têm este comportamento
        
        // Não precisamos fazer nada se o nó já está expandido
        if (expandedNodes.value.has(d.id)) return;
        
        // Adicionar à lista de nós com hover (agora é focus)
        hoveredNodes.value.add(d.id);
        
        // Atualizar textos e cores
        text.text(getNodeText);
        node.attr("fill", getNodeFill);
        
        // Adicionar um indicador visual de focus (borda tracejada)
        node.attr("stroke", n => {
            if (expandedNodes.value.has(n.id)) return "#FFFFFF"; // Expandido tem borda branca sólida
            if (n.id === d.id) return "#AAAAAA"; // Focus tem borda cinza clara
            if (clickedSubNodes.value.has(n.id)) return "#00CCFF"; // Sub_url clicada tem borda azul
            return "none"; // Outros não têm borda
        });
        
        node.attr("stroke-dasharray", n => {
            if (n.id === d.id && !expandedNodes.value.has(n.id)) return "3,3"; // Tracejado para focus
            return "0"; // Sólido para expandidos ou sem borda
        });
    }
    
    // Manipulador para quando o nó perde focus
    function handleNodeBlur(event, d) {
        if (!d.isMainUrl) return;
        
        // Remover da lista de nós com hover (focus)
        hoveredNodes.value.delete(d.id);
        
        // Atualizar textos e cores
        text.text(getNodeText);
        node.attr("fill", getNodeFill);
        
        // Remover indicador visual de focus
        node.attr("stroke", n => {
            if (expandedNodes.value.has(n.id)) return "#FFFFFF"; // Expandido mantém borda branca
            if (clickedSubNodes.value.has(n.id)) return "#00CCFF"; // Sub_url clicada mantém borda azul
            return "none"; // Outros não têm borda
        });
        node.attr("stroke-dasharray", "0"); // Remover tracejado
    }

    // Manipulador para teclas pressionadas enquanto um nó está focado
    function handleKeyDown(event, d) {
        if (!d.isMainUrl) return;
        
        // Expandir/colapsar com Space ou Enter
        if (event.key === " " || event.key === "Enter") {
            event.preventDefault(); // Evitar scroll da página
            toggleNodeExpansion(d);
        }
    }

    // Otimização para o callback de tick
    const ticked = throttle(() => {
        // Não atualizar elementos visuais durante o zoom para melhorar performance
        if (isZooming) return;
        
        link.attr("x1", (d) => d.source.x)
            .attr("y1", (d) => d.source.y)
            .attr("x2", (d) => d.target.x)
            .attr("y2", (d) => d.target.y);

        node.attr("cx", (d) => d.x).attr("cy", (d) => d.y);
        
        // Em dispositivos de baixo desempenho, atualizar textos com menos frequência
        if (!lowPerfModeActive.value || Math.random() < 0.5) {
        text.attr("x", (d) => d.x).attr("y", (d) => d.y);
        }
    }, lowPerfModeActive.value ? 30 : 10); // Throttle mais agressivo em dispositivos de baixo desempenho

    simulation.on("tick", ticked);

    // Armazenar referências globais aos elementos
    activeNodes = node;
    activeLinks = link;
    activeTexts = text;
    activeSimulation = simulation;
    
    // Parar a simulação após um tempo para economizar recursos
    setTimeout(() => {
        simulation.stop();
        // Atualizar posições finais uma última vez
        ticked();
    }, lowPerfModeActive.value ? 2000 : 3000); // Tempo menor em dispositivos de baixo desempenho

    // Ajustar o drag para melhor performance
    function drag(simulation) {
        function dragStarted(event, d) {
            if (!event.active) simulation.alphaTarget(0.3).restart();
            d.fx = d.x;
            d.fy = d.y;
            // Aumentar velocidade de convergência durante arrasto
            simulation.alphaDecay(0.01);
        }

        function dragged(event, d) {
            d.fx = event.x;
            d.fy = event.y;
        }

        function dragEnded(event, d) {
            if (!event.active) simulation.alphaTarget(0);
            d.fx = null;
            d.fy = null;
            // Restaurar velocidade de convergência padrão
            simulation.alphaDecay(0.05);
        }

        return d3
            .drag()
            .on("start", dragStarted)
            .on("drag", dragged)
            .on("end", dragEnded);
    }

    // Armazenar referência global à função ticked
    activeTicked = ticked;

    // Após a simulação inicial ter estabilizado um pouco, iniciar a animação de crescimento
    setTimeout(() => {
        // Certifique-se de que todos os nós tenham posições calculadas primeiro
        if (!simulation.alpha()) {
            simulation.alpha(0.1).restart(); // Dar um pequeno impulso à simulação
            simulation.tick(10); // Forçar algumas atualizações
        }
        
        if (animationEnabled.value) {
            animateGraphGrowth(nodes, links, parentMap, childrenMap, node, link, text, simulation, ticked, width, height);
        } else {
            // Se animação desativada, mostrar todos os elementos com uma transição rápida
            node.transition()
                .duration(300)
                .style("opacity", 1);
                
            text.transition()
                .duration(300)
                .style("opacity", 1);
                
            link.transition()
                .duration(300)
                .style("opacity", 1);
        }
    }, 800); // Aumentar o tempo para permitir que a simulação estabilize mais
}

// Função para animar o crescimento do grafo (movida para o escopo global)
function animateGraphGrowth(nodes, links, parentMap, childrenMap, node, link, text, simulation, ticked, width, height) {
    // Primeiro, verifique se todos os nós têm posições definidas
    nodes.forEach(n => {
        if (!n.x || !n.y) {
            n.x = width / 2 + (Math.random() * 100 - 50);
            n.y = height / 2 + (Math.random() * 100 - 50);
        }
    });
    
    // Mapa para armazenar a profundidade de cada nó
    const nodeDepthMap = new Map();
    
    // Calcular a profundidade de cada nó
    function calculateDepth() {
        // Iniciar com o nó raiz
        let rootNode = nodes.find(n => !parentMap.has(n.id));
        if (!rootNode && nodes.length > 0) rootNode = nodes[0]; // Fallback
        
        if (!rootNode) return; // Não há nós
        
        // Definir nível 0 para raiz
        nodeDepthMap.set(rootNode.id, 0);
        
        // Construir um mapa completo de profundidade
        // Primeiro, lidar com a hierarquia principal usando BFS
        let queue = [rootNode.id];
        let visited = new Set([rootNode.id]);
        
        while (queue.length > 0) {
            const currentId = queue.shift();
            const currentDepth = nodeDepthMap.get(currentId);
            
            // Encontrar todos os filhos deste nó
            const children = childrenMap.get(currentId) || [];
            
            // Atribuir profundidade aos filhos e adicioná-los à fila
            children.forEach(childId => {
                if (!visited.has(childId)) {
                    nodeDepthMap.set(childId, currentDepth + 1);
                    queue.push(childId);
                    visited.add(childId);
                }
            });
        }
        
        // Segundo passo: garantir que todos os nós tenham uma profundidade
        // Isso é especialmente importante para nós isolados ou não conectados diretamente
        nodes.forEach(node => {
            if (!nodeDepthMap.has(node.id)) {
                const parentId = parentMap.get(node.id);
                if (parentId && nodeDepthMap.has(parentId)) {
                    // Atribuir profundidade baseada no pai
                    nodeDepthMap.set(node.id, nodeDepthMap.get(parentId) + 1);
                } else {
                    // Para nós sem pai ou com pai desconhecido, usar profundidade máxima + 1
                    const maxDepth = Math.max(0, ...Array.from(nodeDepthMap.values()));
                    nodeDepthMap.set(node.id, maxDepth + 1);
                }
            }
        });
    }
    
    // Calcular profundidade para todos os nós
    calculateDepth();
    
    // Verificação adicional: garantir que cada nó tenha uma profundidade
    nodes.forEach(n => {
        if (!nodeDepthMap.has(n.id)) {
            nodeDepthMap.set(n.id, 1); // Atribuir um valor padrão
        }
    });
    
    // Determinar a profundidade máxima
    const maxDepth = Math.max(0, ...Array.from(nodeDepthMap.values()));
    
    // Agrupar nós por profundidade para facilitar a animação
    const nodesByDepth = new Map();
    for (let d = 0; d <= maxDepth; d++) {
        nodesByDepth.set(d, nodes.filter(n => nodeDepthMap.get(n.id) === d).map(n => n.id));
    }
    
    // Duração da animação por nível (ms)
    const durationPerLevel = 400;
    
    // Função para aplicar fade-in aos elementos com base na profundidade
    function animateElementsAtDepth(depth) {
        // Se exceder a profundidade máxima, parar
        if (depth > maxDepth) {
            // Forçar uma atualização final após o último nível
            setTimeout(ticked, 100);
            return;
        }
        
        // Garantir que temos os IDs dos nós para este nível
        const nodeIds = nodesByDepth.get(depth) || [];
        
        if (nodeIds.length === 0) {
            // Se não há nós neste nível, passar para o próximo
            setTimeout(() => animateElementsAtDepth(depth + 1), 100);
            return;
        }
        
        // Forçar uma atualização no ticked para garantir que todos os nós estejam visíveis
        ticked();
        
        // Animar nós com efeito de crescimento e pulsação
        node.filter(d => nodeIds.includes(d.id))
            .attr("r", 0) // Começar com raio 0
            .style("opacity", 0.5) // Começar parcialmente transparente
            .transition()
            .duration(300)
            .style("opacity", 1)
            .attr("r", d => d.isMainUrl ? 12 : 6) // Crescer até o tamanho final
            .on("end", function(d) {
                // Adicionar efeito de pulsação se for URL principal
                if (d.isMainUrl) {
                    d3.select(this)
                        .transition()
                        .duration(400)
                        .attr("r", 14) // Expandir um pouco
                        .transition()
                        .duration(400)
                        .attr("r", 12); // Voltar ao tamanho normal
                }
            });
            
        // Animar textos com fade-in
        text.filter(d => nodeIds.includes(d.id))
            .style("opacity", 0)
            .transition()
            .delay(200) // Pequeno atraso para aparecer após o nó
            .duration(300)
            .style("opacity", 1);
            
        // Animar links com efeito de "crescimento" a partir da origem
        link.filter(d => {
            // Link é visível se tanto o alvo quanto a fonte estão em níveis <= depth
            const sourceDepth = nodeDepthMap.get(d.source.id) || 0;
            const targetDepth = nodeDepthMap.get(d.target.id) || 0;
            return Math.max(sourceDepth, targetDepth) === depth;
        })
        .style("opacity", 0.5)
        .attr("stroke-dasharray", function() {
            const length = this.getTotalLength();
            return `${length} ${length}`;
        })
        .attr("stroke-dashoffset", function() {
            return this.getTotalLength();
        })
        .transition()
        .duration(500)
        .style("opacity", 1)
        .attr("stroke-dashoffset", 0) // Animar o traço
        .on("end", function() {
            // Remover o traço após a animação
            d3.select(this).attr("stroke-dasharray", null);
        });
        
        // Adicionar pequenos efeitos de movimento aleatório (vibração)
        node.filter(d => nodeIds.includes(d.id))
            .each(function(d) {
                if (!d.isMainUrl) { // Apenas para sub_urls para não afetar muito o layout
                    const element = d3.select(this);
                    const originalX = d.x;
                    const originalY = d.y;
                    
                    // Pequena vibração aleatória
                    element.transition()
                        .duration(200)
                        .attr("cx", originalX + (Math.random() * 6 - 3))
                        .attr("cy", originalY + (Math.random() * 6 - 3))
                        .transition()
                        .duration(200)
                        .attr("cx", originalX)
                        .attr("cy", originalY);
                }
            });
        
        // Forçar outra atualização do ticked após a animação
        setTimeout(ticked, 300);
        
        // Agendar a próxima etapa com um pequeno atraso
        if (depth < maxDepth) {
            setTimeout(() => {
                animateElementsAtDepth(depth + 1);
            }, durationPerLevel);
        }
    }
    
    // Após a animação principal, verificar e mostrar nós que possam ter ficado invisíveis
    const ensureAllNodesVisible = () => {
        // Forçar atualização do layout para garantir posições corretas
        simulation.alpha(0.1).restart();
        simulation.tick(10);
        ticked();
        
        // Garantir que todos os nós estejam visíveis
        // Verificar por opacidade baixa OU raio zero
        const invisibleNodes = node.filter(function(d) {
            const opacity = parseFloat(d3.select(this).style("opacity"));
            const radius = parseFloat(d3.select(this).attr("r"));
            return opacity < 0.9 || radius < 1 || (!d.isMainUrl && radius === 0);
        });
        
        // Verificação adicional para sub_URLs
        const subUrlNodes = node.filter(d => !d.isMainUrl);
        
        // Verificar e tornar visíveis todos os nós
        invisibleNodes
            .transition()
            .duration(300)
            .style("opacity", 1)
            .attr("r", d => d.isMainUrl ? 12 : 6);
        
        // Garantir que todas as sub_URLs estejam visíveis
        subUrlNodes
            .transition()
            .duration(300)
            .style("opacity", 1)
            .attr("r", 6);
        
        // Verificar e mostrar textos invisíveis
        const invisibleTexts = text.filter(function() {
            return parseFloat(d3.select(this).style("opacity")) < 0.9;
        });
        
        invisibleTexts
            .transition()
            .duration(300)
            .style("opacity", 1);
        
        // Verificar e mostrar links invisíveis
        const invisibleLinks = link.filter(function() {
            return parseFloat(d3.select(this).style("opacity")) < 0.9;
        });
        
        invisibleLinks
            .transition()
            .duration(300)
            .style("opacity", 1);
        
        // Forçar mais uma atualização para finalizar
        setTimeout(ticked, 100);
    };
    
    // Iniciar a animação a partir do nível 0
    animateElementsAtDepth(0);
    
    // Forçar várias atualizações do layout durante toda a animação
    const tickInterval = setInterval(() => {
        ticked();
    }, 300);
    
    // Limpar o intervalo após a animação completa e verificar todos os nós
    setTimeout(() => {
        clearInterval(tickInterval);
        
        // Executar múltiplos ticks da simulação para garantir posições estáveis
        for (let i = 0; i < 15; i++) {
            simulation.tick();
        }
        ticked();
        
        // Pequeno atraso adicional para garantir que todos os nós foram processados
        setTimeout(() => {
            // Verificar se há nós invisíveis e torná-los visíveis
            ensureAllNodesVisible();
            
            // Forçar atualização final para garantir posições corretas
            setTimeout(ticked, 300);
        }, 500);
    }, (maxDepth + 1) * durationPerLevel + 1000);
}

// Função para ativar/desativar o modo de baixo desempenho e redesenhar o grafo
function toggleLowPerfMode() {
    manualLowPerfMode.value = !manualLowPerfMode.value;
    lowPerfModeActive.value = isLowPerfDevice.value || manualLowPerfMode.value;
    
    // Redesenhar o grafo com as novas configurações
    const graphContainer = document.querySelector('.graph-container');
    if (graphContainer) {
        const legendElement = document.querySelector('.legend');
        const indicatorElement = document.querySelector('.touch-indicator');
        const performanceToggleElement = document.querySelector('.performance-toggle');
        const animationToggleElement = document.querySelector('.animation-toggle');
        const restartButtonElement = document.querySelector('.restart-button');
        
        // Preservar os elementos de UI existentes
        graphContainer.innerHTML = '<svg id="graph"></svg>';
        if (legendElement) graphContainer.appendChild(legendElement);
        if (indicatorElement) graphContainer.appendChild(indicatorElement);
        if (performanceToggleElement) graphContainer.appendChild(performanceToggleElement);
        if (animationToggleElement) graphContainer.appendChild(animationToggleElement);
        if (restartButtonElement) graphContainer.appendChild(restartButtonElement);
        
        drawGraph();
    }
}

// Função para ativar/desativar a animação de crescimento
function toggleAnimation() {
    animationEnabled.value = !animationEnabled.value;
    localStorage.setItem('graph-animation-enabled', animationEnabled.value);
    
    // Redesenhar o grafo com a nova configuração de animação
    const graphContainer = document.querySelector('.graph-container');
    if (graphContainer) {
        const legendElement = document.querySelector('.legend');
        const indicatorElement = document.querySelector('.touch-indicator');
        const performanceToggleElement = document.querySelector('.performance-toggle');
        const animationToggleElement = document.querySelector('.animation-toggle');
        const restartButtonElement = document.querySelector('.restart-button');
        
        // Preservar os elementos de UI existentes
        graphContainer.innerHTML = '<svg id="graph"></svg>';
        if (legendElement) graphContainer.appendChild(legendElement);
        if (indicatorElement) graphContainer.appendChild(indicatorElement);
        if (performanceToggleElement) graphContainer.appendChild(performanceToggleElement);
        if (animationToggleElement) graphContainer.appendChild(animationToggleElement);
        if (restartButtonElement && animationEnabled.value) graphContainer.appendChild(restartButtonElement);
        
        drawGraph();
    }
}

// Função para reiniciar a animação sem redesenhar o grafo inteiro
function restartAnimation() {
    if (!animationEnabled.value) {
        // Se a animação estiver desativada, ativá-la primeiro
        animationEnabled.value = true;
        localStorage.setItem('graph-animation-enabled', 'true');
    }
    
    // Verificar se temos elementos do grafo disponíveis
    if (!activeNodes || !activeLinks || !activeTexts || !activeSimulation) {
        console.warn("Elementos do grafo não disponíveis para reiniciar a animação");
        return;
    }
    
    // Resetar os elementos para ficarem invisíveis
    activeNodes.interrupt().style("opacity", 0);
    activeLinks.interrupt().style("opacity", 0);
    activeTexts.interrupt().style("opacity", 0);
    
    // Resetar raio dos nós para o valor inicial
    activeNodes.attr("r", 0);
    
    // Reiniciar a simulação para recalcular posições
    activeSimulation.alpha(0.3).restart();
    activeSimulation.tick(10);
    
    // Iniciar a animação
    setTimeout(() => {
        // Recuperar as seleções d3 atuais dos elementos
        const nodes = activeNodes.data();
        const links = activeLinks.data();
        
        // Chamar a função de animação com todos os parâmetros necessários
        animateGraphGrowth(
            nodes, 
            links, 
            activeParentMap, 
            activeChildrenMap, 
            activeNodes, 
            activeLinks, 
            activeTexts, 
            activeSimulation, 
            activeTicked,
            activeWidth,
            activeHeight
        );
    }, 100);
}
</script>

<template>
    <div class="graph-container">
        <svg id="graph" ref="svgRef"></svg>
        <div class="legend">
            <div class="legend-item">
                <div class="legend-color main-url"></div>
                <span>URL raiz</span>
            </div>
            <div class="legend-item">
                <div class="legend-color sub-url"></div>
                <span>URL sem sub-links</span>
            </div>
            <div class="legend-item">
                <div class="legend-color main-url expanded"></div>
                <span>Expandida</span>
            </div>
            <div class="legend-item">
                <div class="legend-color main-url hovered"></div>
                <span>Com foco</span>
            </div>
            <div class="legend-item">
                <div class="legend-color highlighted-sub"></div>
                <span>Sub-URL destacada</span>
            </div>
            <div class="legend-item">
                <div class="legend-color clicked-sub"></div>
                <span>Sub-URL clicada</span>
            </div>
            <div class="legend-info">
                Clique: foco | Duplo clique: expandir | ESPAÇO/ENTER: expandir
            </div>
            <div class="legend-info">
                Clique nas sub-URLs (cinzas/azuis) para mostrar/ocultar seu nome
            </div>
            <div class="legend-info" v-if="animationEnabled">
                Animação molecular ativada: observe o grafo crescer organicamente
            </div>
        </div>
        
        <!-- Indicador para dispositivo touch -->
        <div class="touch-indicator" v-if="isTouchDevice">
            <span>Toque duplo para expandir</span>
            <div class="keyboard-keys">
                <span class="key">⎵</span>
                <span class="key">↩</span>
            </div>
        </div>
        
        <!-- Botão para alternar modo de desempenho -->
        <div class="performance-toggle" @click="toggleLowPerfMode">
            <span v-if="manualLowPerfMode">Modo de alto desempenho</span>
            <span v-else>Modo de baixo desempenho</span>
            <div class="toggle-switch" :class="{ active: manualLowPerfMode }">
                <div class="toggle-knob"></div>
            </div>
        </div>
        
        <!-- Botão para alternar animação -->
        <div class="animation-toggle" @click="toggleAnimation">
            <span v-if="animationEnabled">Animação: Ligada</span>
            <span v-else>Animação: Desligada</span>
            <div class="toggle-switch" :class="{ active: animationEnabled }">
                <div class="toggle-knob"></div>
            </div>
        </div>
        
        <!-- Botão para reiniciar a animação -->
        <div class="restart-button" @click="restartAnimation" v-if="animationEnabled" title="Reiniciar animação">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4C7.58 4 4.01 7.58 4.01 12C4.01 16.42 7.58 20 12 20C15.73 20 18.84 17.45 19.73 14H17.65C16.83 16.33 14.61 18 12 18C8.69 18 6 15.31 6 12C6 8.69 8.69 6 12 6C13.66 6 15.14 6.69 16.22 7.78L13 11H20V4L17.65 6.35Z" fill="white"/>
            </svg>
        </div>
    </div>
</template>

<style scoped>
.graph-container {
    width: 100%;
    height: 100vh;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: #1e1e1e;
    border-radius: 10px;
}

svg {
    width: 100%;
    height: 100%;
}

text {
    fill: white;
}

.legend {
    position: absolute;
    bottom: 20px;
    right: 20px;
    background-color: rgba(0, 0, 0, 0.7);
    padding: 10px;
    border-radius: 5px;
    color: white;
}

.legend-item {
    display: flex;
    align-items: center;
    margin-bottom: 5px;
}

.legend-color {
    width: 15px;
    height: 15px;
    border-radius: 50%;
    margin-right: 10px;
}

.main-url {
    background-color: #FF4444;
}

.sub-url {
    background-color: #888888;
}

.expanded {
    border: 2px solid white;
}

.hovered {
    border: 2px dashed #AAAAAA;
}

.highlighted-sub {
    background-color: #AADDFF;
}

.clicked-sub {
    background-color: #888888;
    border: 2px solid #00CCFF;
}

.legend-info {
    font-size: 12px;
    margin-top: 10px;
    color: #cccccc;
}

/* Estilo para a borda de foco acessível */
circle:focus {
    outline: none; /* Remove a borda de foco padrão */
}

.touch-indicator {
    position: absolute;
    top: 20px;
    left: 50%;
    transform: translateX(-50%);
    background-color: rgba(0, 0, 0, 0.7);
    color: white;
    padding: 10px 15px;
    border-radius: 20px;
    font-size: 14px;
    animation: pulse 2s infinite;
    text-align: center;
}

.keyboard-keys {
    display: flex;
    justify-content: center;
    margin-top: 8px;
}

.key {
    display: inline-block;
    background-color: #444;
    border: 1px solid #666;
    border-radius: 4px;
    padding: 2px 8px;
    margin: 0 5px;
    font-size: 18px;
}

@keyframes pulse {
    0% { opacity: 0.8; }
    50% { opacity: 1; }
    100% { opacity: 0.8; }
}

.performance-toggle, .animation-toggle {
    position: absolute;
    background-color: rgba(0, 0, 0, 0.7);
    color: white;
    padding: 10px 15px;
    border-radius: 20px;
    font-size: 14px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 200px;
}

.performance-toggle {
    top: 20px;
    right: 20px;
}

.animation-toggle {
    top: 70px;
    right: 20px;
}

.restart-button {
    position: absolute;
    top: 20px;
    left: 20px;
    background-color: rgba(0, 0, 0, 0.7);
    color: white;
    padding: 8px;
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.3);
    transition: all 0.2s ease;
}

.restart-button:hover {
    transform: scale(1.1);
    background-color: rgba(0, 0, 0, 0.8);
}

.restart-button svg {
    animation: spin 1.5s infinite linear paused;
}

.restart-button:hover svg {
    animation-play-state: running;
}

@keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
}

.toggle-switch {
    width: 40px;
    height: 20px;
    background-color: #444;
    border-radius: 10px;
    position: relative;
    margin-left: 10px;
}

.toggle-knob {
    width: 16px;
    height: 16px;
    background-color: white;
    border-radius: 50%;
    position: absolute;
    top: 2px;
    left: 2px;
    transition: transform 0.3s;
}

.toggle-switch.active .toggle-knob {
    transform: translateX(20px);
}
</style>
