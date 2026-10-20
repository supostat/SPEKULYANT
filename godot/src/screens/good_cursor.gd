class_name GoodCursor
extends RefCounted
## Товар толкучки, на котором стоят график цены и цены по городам: выбор держится за идентификатор
## и переживает новые снимки, а товар, пропавший с толкучки, уступает место первому.

## Идентификатор товара; пусто — первый товар толкучки.
var good := ""


## Строка выбранного товара.
func row(goods: Array) -> Dictionary:
	var index := goods.find_custom(func(shown: Dictionary) -> bool: return shown["good"] == good)
	return goods[maxi(index, 0)]


## Соседний товар: выше при `step` = −1, ниже при +1; у краёв толкучки выбор стоит.
func move(goods: Array, step: int) -> void:
	var index := goods.find(row(goods)) + step
	good = goods[clampi(index, 0, goods.size() - 1)]["good"]
