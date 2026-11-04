#include <metal_stdlib>
using namespace metal;
#pragma clang fp contract(off)

// tpe_coherence_int.metal — INTEGER-ONLY coherence kernel (2026-10-27, spec §9 gate 1).
// NO FLOAT ops. Port of TorPhaseEncoder::evolve_vector with: CORDIC trig (pi-unit native),
// Q32 unwrapped/amp/shell, om 2^-e chain + POW2FRAC[1025] table, integer log2 (msb+LOG2MANT),
// exact-T ha reduction + (-1)^q guard, Q32 MACs. Declared-reference tables (FoldAttention).
// INPUT BOUNDARY (preregistered): xs host-f32, converted ONCE per element via rint(x*2^31).
// No other float op in this file (grep-verified).

constant long TPE_OMEGA_F = 214748365l;    // 0.1
constant long TPE_OMEGA_S = 107374182l;    // 0.05
constant long TPE_K_F = 644245094l;    // 0.3
constant long TPE_K_S = 322122547l;    // 0.15
constant long TPE_PHASE_SCALE = 322122547l;    // 0.15
constant long Q32_INV_TAU = 341782638l;    // round(2^31/TAU)
constant long Q32_LN2 = 1488522236l;    // round(ln2*2^31)
constant long Q32_INV_SQRT2 = 1518500250l;    // round(2^31/sqrt(2)): phi scale
constant long Q32_C096 = 2061584302l;    // 0.96
constant long Q32_C004 = 85899346l;    // 0.04
constant long Q32_HALF = 1073741824l;    // 0.5
constant long Q32_W0 = 1975684956l;    // 0.92
constant long Q32_W1 = 171798692l;    // 0.08
constant long Q32_RMIN = 472446403l;    // 0.22
constant long Q32_RSPAN = 1460288881l;    // 0.68
constant long Q32_A1 = 751619277l;    // 0.35
constant long Q32_A2 = 1395864371l;    // 0.65
constant long Q32_B1 = 601295421l;    // 0.28
constant long Q32_B2 = 429496730l;    // 0.20
constant long Q32_C005 = 107374182l;    // 0.05
constant long Q32_C038 = 816043786l;    // 0.38
constant long Q32_ONE = 2147483648l;    // 1.0
constant long Q32_T_Q32 = 8339155913l;    // round(T*2^31), exact-T golden period
// (two-part T_hi superseded: single exact subtract (ab<<31)-qq*T is exact i64, guard handles the rest)
constant long Q32_T_HALF = 4169577957l;    // round(T/2), remainder-guard bound
constant long Q32_INV_PI = 683565276l;    // round(2^31/pi)
constant long Q32_PHI_OVER_PI = 1106031849l;    // round(phi/pi*2^31), phi=1.61803398875 (float-kernel parity fix 2026-10-27: ha = ab_red*phi + r*pi/4, the xphi was dropped in the int port)
constant long Q32_LOG2_10000 = 28535145054l;    // round(log2(10000)*2^31)
constant long Q32_PARM1 = -1327217885l;    // -0.61803398875 parity

constant long POW2FRAC[1025] = {
    2147483648, 2146030505, 2144578345, 2143127168,
    2141676973, 2140227759, 2138779525, 2137332272,
    2135885998, 2134440703, 2132996386, 2131553046,
    2130110682, 2128669295, 2127228883, 2125789446,
    2124350982, 2122913493, 2121476975, 2120041430,
    2118606857, 2117173254, 2115740621, 2114308957,
    2112878262, 2111448536, 2110019777, 2108591984,
    2107165158, 2105739297, 2104314402, 2102890470,
    2101467502, 2100045496, 2098624453, 2097204372,
    2095785251, 2094367091, 2092949891, 2091533649,
    2090118366, 2088704040, 2087290671, 2085878259,
    2084466803, 2083056301, 2081646755, 2080238161,
    2078830522, 2077423834, 2076018099, 2074613314,
    2073209480, 2071806597, 2070404662, 2069003676,
    2067603638, 2066204548, 2064806404, 2063409206,
    2062012954, 2060617646, 2059223283, 2057829863,
    2056437387, 2055045852, 2053655259, 2052265607,
    2050876895, 2049489123, 2048102290, 2046716396,
    2045331439, 2043947420, 2042564337, 2041182189,
    2039800978, 2038420700, 2037041357, 2035662947,
    2034285470, 2032908925, 2031533312, 2030158629,
    2028784876, 2027412053, 2026040159, 2024669194,
    2023299156, 2021930045, 2020561860, 2019194602,
    2017828268, 2016462860, 2015098375, 2013734813,
    2012372174, 2011010457, 2009649662, 2008289787,
    2006930832, 2005572798, 2004215682, 2002859484,
    2001504204, 2000149841, 1998796395, 1997443864,
    1996092249, 1994741548, 1993391761, 1992042888,
    1990694927, 1989347879, 1988001742, 1986656516,
    1985312200, 1983968794, 1982626297, 1981284708,
    1979944027, 1978604253, 1977265386, 1975927425,
    1974590370, 1973254219, 1971918972, 1970584628,
    1969251188, 1967918650, 1966587013, 1965256278,
    1963926443, 1962597508, 1961269472, 1959942335,
    1958616096, 1957290755, 1955966310, 1954642761,
    1953320108, 1951998350, 1950677487, 1949357517,
    1948038440, 1946720256, 1945402964, 1944086563,
    1942771053, 1941456434, 1940142704, 1938829862,
    1937517909, 1936206844, 1934896666, 1933587375,
    1932278970, 1930971450, 1929664814, 1928359063,
    1927054196, 1925750211, 1924447109, 1923144889,
    1921843549, 1920543091, 1919243512, 1917944813,
    1916646992, 1915350050, 1914053985, 1912758797,
    1911464486, 1910171051, 1908878490, 1907586805,
    1906295993, 1905006055, 1903716990, 1902428797,
    1901141476, 1899855026, 1898569446, 1897284736,
    1896000896, 1894717924, 1893435821, 1892154585,
    1890874216, 1889594713, 1888316077, 1887038305,
    1885761398, 1884485355, 1883210176, 1881935859,
    1880662405, 1879389813, 1878118081, 1876847210,
    1875577199, 1874308048, 1873039755, 1871772321,
    1870505744, 1869240024, 1867975161, 1866711153,
    1865448001, 1864185704, 1862924261, 1861663671,
    1860403934, 1859145050, 1857887018, 1856629837,
    1855373507, 1854118026, 1852863396, 1851609614,
    1850356681, 1849104595, 1847853357, 1846602965,
    1845353420, 1844104720, 1842856865, 1841609855,
    1840363688, 1839118364, 1837873883, 1836630245,
    1835387448, 1834145491, 1832904376, 1831664100,
    1830424663, 1829186065, 1827948305, 1826711383,
    1825475297, 1824240048, 1823005635, 1821772057,
    1820539314, 1819307406, 1818076330, 1816846088,
    1815616678, 1814388100, 1813160354, 1811933438,
    1810707353, 1809482097, 1808257670, 1807034072,
    1805811301, 1804589359, 1803368243, 1802147953,
    1800928489, 1799709850, 1798492036, 1797275046,
    1796058879, 1794843536, 1793629014, 1792415315,
    1791202437, 1789990379, 1788779142, 1787568724,
    1786359126, 1785150346, 1783942384, 1782735239,
    1781528911, 1780323399, 1779118704, 1777914823,
    1776711757, 1775509505, 1774308066, 1773107441,
    1771907628, 1770708627, 1769510437, 1768313058,
    1767116489, 1765920730, 1764725780, 1763531638,
    1762338305, 1761145779, 1759954060, 1758763148,
    1757573041, 1756383740, 1755195243, 1754007551,
    1752820662, 1751634577, 1750449294, 1749264813,
    1748081133, 1746898255, 1745716177, 1744534899,
    1743354420, 1742174740, 1740995858, 1739817774,
    1738640488, 1737463997, 1736288303, 1735113405,
    1733939301, 1732765992, 1731593477, 1730421755,
    1729250827, 1728080690, 1726911345, 1725742792,
    1724575029, 1723408057, 1722241874, 1721076480,
    1719911875, 1718748058, 1717585029, 1716422786,
    1715261330, 1714100660, 1712940775, 1711781675,
    1710623359, 1709465828, 1708309079, 1707153113,
    1705997930, 1704843528, 1703689907, 1702537067,
    1701385007, 1700233727, 1699083225, 1697933502,
    1696784557, 1695636390, 1694489000, 1693342385,
    1692196547, 1691051484, 1689907196, 1688763683,
    1687620943, 1686478976, 1685337782, 1684197360,
    1683057710, 1681918831, 1680780723, 1679643385,
    1678506817, 1677371017, 1676235986, 1675101724,
    1673968228, 1672835500, 1671703538, 1670572342,
    1669441912, 1668312247, 1667183346, 1666055209,
    1664927835, 1663801224, 1662675375, 1661550289,
    1660425963, 1659302399, 1658179594, 1657057550,
    1655936265, 1654815738, 1653695970, 1652576959,
    1651458706, 1650341209, 1649224469, 1648108484,
    1646993254, 1645878779, 1644765058, 1643652091,
    1642539877, 1641428415, 1640317706, 1639207748,
    1638098541, 1636990085, 1635882379, 1634775422,
    1633669214, 1632563755, 1631459044, 1630355081,
    1629251865, 1628149395, 1627047671, 1625946692,
    1624846459, 1623746970, 1622648225, 1621550224,
    1620452965, 1619356449, 1618260675, 1617165643,
    1616071351, 1614977800, 1613884989, 1612792918,
    1611701585, 1610610991, 1609521135, 1608432016,
    1607343634, 1606255989, 1605169080, 1604082906,
    1602997467, 1601912763, 1600828793, 1599745556,
    1598663052, 1597581281, 1596500241, 1595419934,
    1594340357, 1593261510, 1592183394, 1591106008,
    1590029350, 1588953421, 1587878220, 1586803746,
    1585730000, 1584656980, 1583584686, 1582513118,
    1581442275, 1580372156, 1579302762, 1578234091,
    1577166143, 1576098919, 1575032416, 1573966635,
    1572901575, 1571837235, 1570773616, 1569710717,
    1568648537, 1567587076, 1566526333, 1565466307,
    1564406999, 1563348408, 1562290533, 1561233374,
    1560176931, 1559121202, 1558066187, 1557011887,
    1555958300, 1554905425, 1553853264, 1552801814,
    1551751076, 1550701048, 1549651732, 1548603125,
    1547555228, 1546508040, 1545461560, 1544415789,
    1543370725, 1542326369, 1541282719, 1540239775,
    1539197537, 1538156005, 1537115177, 1536075053,
    1535035634, 1533996917, 1532958904, 1531921593,
    1530884983, 1529849076, 1528813869, 1527779362,
    1526745556, 1525712449, 1524680042, 1523648333,
    1522617322, 1521587009, 1520557392, 1519528473,
    1518500250, 1517472723, 1516445891, 1515419753,
    1514394310, 1513369561, 1512345506, 1511322143,
    1510299473, 1509277495, 1508256209, 1507235613,
    1506215708, 1505196493, 1504177968, 1503160132,
    1502142985, 1501126526, 1500110755, 1499095672,
    1498081275, 1497067565, 1496054540, 1495042201,
    1494030547, 1493019578, 1492009293, 1490999691,
    1489990772, 1488982537, 1487974983, 1486968111,
    1485961921, 1484956411, 1483951582, 1482947433,
    1481943963, 1480941172, 1479939060, 1478937626,
    1477936870, 1476936791, 1475937388, 1474938662,
    1473940611, 1472943236, 1471946536, 1470950510,
    1469955159, 1468960480, 1467966475, 1466973143,
    1465980482, 1464988494, 1463997176, 1463006530,
    1462016553, 1461027247, 1460038610, 1459050642,
    1458063343, 1457076711, 1456090748, 1455105451,
    1454120821, 1453136858, 1452153560, 1451170927,
    1450188960, 1449207657, 1448227018, 1447247043,
    1446267730, 1445289081, 1444311093, 1443333768,
    1442357104, 1441381100, 1440405757, 1439431074,
    1438457051, 1437483687, 1436510981, 1435538933,
    1434567544, 1433596811, 1432626736, 1431657317,
    1430688553, 1429720446, 1428752993, 1427786196,
    1426820052, 1425854562, 1424889725, 1423925542,
    1422962010, 1421999131, 1421036903, 1420075327,
    1419114401, 1418154125, 1417194499, 1416235523,
    1415277195, 1414319516, 1413362485, 1412406101,
    1411450365, 1410495275, 1409540832, 1408587035,
    1407633882, 1406681375, 1405729513, 1404778294,
    1403827719, 1402877788, 1401928499, 1400979853,
    1400031848, 1399084485, 1398137763, 1397191682,
    1396246240, 1395301439, 1394357277, 1393413754,
    1392470869, 1391528622, 1390587013, 1389646041,
    1388705706, 1387766007, 1386826944, 1385888516,
    1384950723, 1384013565, 1383077041, 1382141151,
    1381205894, 1380271270, 1379337279, 1378403919,
    1377471191, 1376539094, 1375607628, 1374676792,
    1373746586, 1372817010, 1371888062, 1370959743,
    1370032052, 1369104989, 1368178554, 1367252745,
    1366327563, 1365403006, 1364479076, 1363555770,
    1362633090, 1361711033, 1360789601, 1359868792,
    1358948606, 1358029043, 1357110102, 1356191783,
    1355274085, 1354357009, 1353440552, 1352524716,
    1351609500, 1350694903, 1349780925, 1348867565,
    1347954824, 1347042700, 1346131193, 1345220303,
    1344310030, 1343400372, 1342491330, 1341582903,
    1340675091, 1339767893, 1338861309, 1337955338,
    1337049980, 1336145235, 1335241103, 1334337582,
    1333434672, 1332532374, 1331630686, 1330729608,
    1329829140, 1328929281, 1328030031, 1327131390,
    1326233356, 1325335931, 1324439112, 1323542901,
    1322647296, 1321752297, 1320857903, 1319964115,
    1319070932, 1318178353, 1317286378, 1316395006,
    1315504238, 1314614072, 1313724509, 1312835548,
    1311947188, 1311059430, 1310172272, 1309285714,
    1308399756, 1307514398, 1306629639, 1305745479,
    1304861917, 1303978953, 1303096586, 1302214816,
    1301333643, 1300453067, 1299573086, 1298693701,
    1297814910, 1296936715, 1296059113, 1295182106,
    1294305692, 1293429870, 1292554642, 1291680006,
    1290805962, 1289932509, 1289059647, 1288187376,
    1287315695, 1286444604, 1285574102, 1284704189,
    1283834865, 1282966129, 1282097982, 1281230421,
    1280363448, 1279497061, 1278631261, 1277766046,
    1276901417, 1276037373, 1275173913, 1274311038,
    1273448747, 1272587039, 1271725915, 1270865373,
    1270005413, 1269146035, 1268287239, 1267429024,
    1266571390, 1265714336, 1264857861, 1264001967,
    1263146652, 1262291915, 1261437757, 1260584177,
    1259731174, 1258878748, 1258026900, 1257175628,
    1256324931, 1255474811, 1254625266, 1253776295,
    1252927899, 1252080077, 1251232829, 1250386154,
    1249540052, 1248694523, 1247849566, 1247005180,
    1246161366, 1245318123, 1244475451, 1243633348,
    1242791816, 1241950853, 1241110459, 1240270633,
    1239431376, 1238592687, 1237754566, 1236917011,
    1236080024, 1235243602, 1234407747, 1233572457,
    1232737732, 1231903573, 1231069977, 1230236946,
    1229404479, 1228572575, 1227741233, 1226910455,
    1226080238, 1225250583, 1224421490, 1223592958,
    1222764986, 1221937574, 1221110723, 1220284431,
    1219458698, 1218633524, 1217808908, 1216984850,
    1216161350, 1215338407, 1214516021, 1213694191,
    1212872918, 1212052200, 1211232038, 1210412430,
    1209593378, 1208774879, 1207956934, 1207139543,
    1206322705, 1205506419, 1204690686, 1203875505,
    1203060876, 1202246798, 1201433270, 1200620294,
    1199807867, 1198995990, 1198184662, 1197373884,
    1196563654, 1195753972, 1194944838, 1194136252,
    1193328213, 1192520720, 1191713774, 1190907374,
    1190101520, 1189296211, 1188491447, 1187687228,
    1186883552, 1186080421, 1185277833, 1184475788,
    1183674286, 1182873326, 1182072908, 1181273032,
    1180473697, 1179674903, 1178876649, 1178078936,
    1177281762, 1176485128, 1175689033, 1174893476,
    1174098458, 1173303978, 1172510036, 1171716630,
    1170923762, 1170131430, 1169339634, 1168548374,
    1167757650, 1166967460, 1166177806, 1165388685,
    1164600099, 1163812046, 1163024526, 1162237539,
    1161451085, 1160665163, 1159879773, 1159094914,
    1158310587, 1157526790, 1156743523, 1155960787,
    1155178580, 1154396902, 1153615754, 1152835134,
    1152055042, 1151275478, 1150496441, 1149717932,
    1148939949, 1148162493, 1147385563, 1146609159,
    1145833280, 1145057926, 1144283097, 1143508792,
    1142735011, 1141961754, 1141189020, 1140416809,
    1139645120, 1138873953, 1138103309, 1137333186,
    1136563583, 1135794502, 1135025941, 1134257900,
    1133490379, 1132723378, 1131956895, 1131190931,
    1130425485, 1129660557, 1128896147, 1128132254,
    1127368878, 1126606018, 1125843675, 1125081848,
    1124320536, 1123559739, 1122799457, 1122039689,
    1121280436, 1120521696, 1119763470, 1119005757,
    1118248556, 1117491868, 1116735692, 1115980028,
    1115224875, 1114470233, 1113716102, 1112962481,
    1112209370, 1111456768, 1110704676, 1109953093,
    1109202018, 1108451451, 1107701393, 1106951842,
    1106202798, 1105454261, 1104706230, 1103958706,
    1103211687, 1102465174, 1101719167, 1100973664,
    1100228665, 1099484170, 1098740180, 1097996693,
    1097253708, 1096511227, 1095769248, 1095027771,
    1094286796, 1093546322, 1092806349, 1092066877,
    1091327906, 1090589434, 1089851462, 1089113990,
    1088377016, 1087640541, 1086904565, 1086169087,
    1085434106, 1084699622, 1083965636, 1083232146,
    1082499153, 1081766656, 1081034654, 1080303147,
    1079572136, 1078841619, 1078111597, 1077382068,
    1076653033, 1075924492, 1075196443, 1074468888,
    1073741824
};

constant long LOG2MANT[256] = {
    0, 12078627, 24110347, 36095523,
    48034513, 59927671, 71775349, 83577893,
    95335645, 107048945, 118718126, 130343521,
    141925456, 153464255, 164960239, 176413723,
    187825021, 199194443, 210522295, 221808880,
    233054496, 244259442, 255424009, 266548488,
    277633165, 288678325, 299684247, 310651211,
    321579490, 332469358, 343321082, 354134928,
    364911162, 375650043, 386351829, 397016776,
    407645136, 418237160, 428793095, 439313187,
    449797678, 460246807, 470660814, 481039932,
    491384396, 501694436, 511970279, 522212153,
    532420281, 542594885, 552736183, 562844395,
    572919734, 582962413, 592972645, 602950638,
    612896598, 622810731, 632693241, 642544327,
    652364189, 662153025, 671911030, 681638398,
    691335320, 701001986, 710638585, 720245302,
    729822324, 739369832, 748888009, 758377033,
    767837083, 777268336, 786670965, 796045145,
    805391046, 814708840, 823998694, 833260775,
    842495250, 851702282, 860882034, 870034667,
    879160341, 888259214, 897331443, 906377184,
    915396590, 924389816, 933357012, 942298328,
    951213914, 960103918, 968968484, 977807760,
    986621888, 995411012, 1004175273, 1012914810,
    1021629764, 1030320272, 1038986470, 1047628495,
    1056246482, 1064840562, 1073410869, 1081957534,
    1090480686, 1098980456, 1107456970, 1115910356,
    1124340739, 1132748245, 1141132997, 1149495118,
    1157834731, 1166151954, 1174446910, 1182719716,
    1190970490, 1199199350, 1207406412, 1215591791,
    1223755601, 1231897955, 1240018966, 1248118746,
    1256197405, 1264255053, 1272291800, 1280307752,
    1288303019, 1296277705, 1304231918, 1312165761,
    1320079339, 1327972754, 1335846110, 1343699509,
    1351533050, 1359346835, 1367140963, 1374915531,
    1382670639, 1390406384, 1398122861, 1405820167,
    1413498396, 1421157644, 1428798003, 1436419566,
    1444022426, 1451606675, 1459172403, 1466719700,
    1474248656, 1481759361, 1489251901, 1496726366,
    1504182841, 1511621414, 1519042169, 1526445193,
    1533830570, 1541198383, 1548548716, 1555881652,
    1563197273, 1570495661, 1577776895, 1585041058,
    1592288229, 1599518487, 1606731910, 1613928578,
    1621108567, 1628271955, 1635418819, 1642549234,
    1649663276, 1656761020, 1663842541, 1670907913,
    1677957208, 1684990500, 1692007863, 1699009366,
    1705995083, 1712965083, 1719919439, 1726858219,
    1733781493, 1740689331, 1747581801, 1754458972,
    1761320910, 1768167684, 1774999361, 1781816006,
    1788617686, 1795404466, 1802176412, 1808933588,
    1815676059, 1822403888, 1829117139, 1835815874,
    1842500157, 1849170050, 1855825614, 1862466912,
    1869094003, 1875706949, 1882305810, 1888890646,
    1895461516, 1902018479, 1908561594, 1915090920,
    1921606515, 1928108435, 1934596739, 1941071483,
    1947532725, 1953980519, 1960414922, 1966835990,
    1973243777, 1979638338, 1986019729, 1992388003,
    1998743213, 2005085414, 2011414658, 2017730999,
    2024034488, 2030325179, 2036603122, 2042868370,
    2049120974, 2055360984, 2061588451, 2067803426,
    2074005959, 2080196099, 2086373895, 2092539398,
    2098692655, 2104833716, 2110962628, 2117079439,
    2123184198, 2129276951, 2135357746, 2141426629
};

constant long CORDIC_GAIN = 1304065748l;
constant long CORDIC_ATAN[32] = {
    536870912, 316933406, 167458907, 85004756,
    42667331, 21354465, 10679838, 5340245,
    2670163, 1335087, 667544, 333772,
    166886, 83443, 41722, 20861,
    10430, 5215, 2608, 1304,
    652, 326, 163, 81,
    41, 20, 10, 5,
    3, 1, 1, 0
};
inline long q32_cordic_sin(long theta) {
    long z = (long)(int)theta;
    if (z == 0) return 0;
    long pi_half = 1l << 30;
    long pi = 1l << 31;
    if (z > pi_half) { z = pi - z; }
    else if (z < -pi_half) { z = -pi - z; }
    long x = CORDIC_GAIN;
    long y = 0l;
    for (int i = 0; i < 32; i++) {
        long x_shift = x >> i;
        long y_shift = y >> i;
        if (z >= 0) {
            x = x - y_shift; y = y + x_shift; z = z - CORDIC_ATAN[i];
        } else {
            x = x + y_shift; y = y - x_shift; z = z + CORDIC_ATAN[i];
        }
    }
    return y;
}
inline long q32_cordic_cos(long theta) {
    return q32_cordic_sin(theta + (1l << 30));
}
// Q32 mul == Rust (a as i128 * b as i128) >> 31 (FLOOR). Carry-folded split,
// VERIFIED 0/500000 vs i128 truth (q32_cordic.metal).
inline long q32_mul(long a, long b) {
    long a_hi = a >> 16; long a_lo = a - (a_hi << 16);
    long b_hi = b >> 16; long b_lo = b - (b_hi << 16);
    long p0 = a_lo * b_lo; long p1 = a_hi * b_lo; long p2 = a_lo * b_hi; long p3 = a_hi * b_hi;
    long mid = p1 + p2;
    long t = (mid << 16) + p0;
    return (p3 << 1) + (t >> 31);
}
// Q32 div (Rust fixed.rs::div semantics; certified range: au<<31 fits, see tpe_phase).
inline long q32_div(long a, long b) {
    if (b == 0) return (a == 0) ? 0l : ((a > 0) ? 9223372036854775807l : (-9223372036854775807l - 1l));
    int aneg = (a < 0) ? 1 : 0;
    int bneg = (b < 0) ? 1 : 0;
    ulong au = (ulong)(aneg ? -a : a);
    ulong bu = (ulong)(bneg ? -b : b);
    ulong num = au << 31;
    long q = (long)(num / bu);
    return ((aneg != bneg) && q != 0) ? -q : q;
}
// Integer sqrt, msb start (VERIFIED: converges everywhere incl. tiny sums).
// msb position of v>0 (binary search, deterministic trip count).
inline int q32_msb(ulong v) {
    int b = 0;
    if (v >= (1ul << 32)) { b += 32; v >>= 32; }
    if (v >= (1ul << 16)) { b += 16; v >>= 16; }
    if (v >= (1ul << 8)) { b += 8; v >>= 8; }
    if (v >= (1ul << 4)) { b += 4; v >>= 4; }
    if (v >= (1ul << 2)) { b += 2; v >>= 2; }
    if (v >= (1ul << 1)) { b += 1; }
    return b;
}
// Round-to-nearest Q32 division (CPU from_f64 parity, 2026-10-28):
// q = round(a/b) in Q32, same range contract as q32_div ((a<<31) must fit).
// Bitwise vs Rust Q32_31::from_f64 for drive-path magnitudes (|a|<|b|, same sign
// domains as the kernel drive sites). No float.
inline long q32_div_round(long a, long b) {
    // CPU-PARITY (2026-10-28): Rust (i128<<31)/i128 truncates toward ZERO
    // (not floor). from_f64 rounds the true quotient to nearest, so the kernel
    // must compute trunc() then round-half-away-from-zero on the REMAINDER
    // (remainder carries the dividend's sign under C-truncation). q32_div keeps
    // floor semantics (matches i128>>31 arithmetic-shift sites); this helper is
    // ONLY for the from_f64-parity drive sites. No float.
    if (b == 0) return (a == 0) ? 0l : ((a > 0) ? 9223372036854775807l : (-9223372036854775807l - 1l));
    int neg = ((a < 0) != (b < 0)) ? 1 : 0;
    ulong au = (ulong)((a < 0) ? -a : a);
    ulong bu = (ulong)((b < 0) ? -b : b);
    ulong num = au << 31;
    ulong q = num / bu;
    ulong rem = num % bu;
    if ((rem << 1) >= bu) q += 1ul;   // rem<bu so rem<<1 fits (bu<=2^62 here)
    long qs = (long)q;
    return (neg && qs != 0) ? -qs : qs;
}
// Q32 division for LARGE operands (symmetric msb downscale, VERIFIED pattern):
// q = a/b in Q32 with a,b arbitrary magnitude: shift both right by k so the
// (a<<31) fits, i.e. k = msb(max(a,b,2^31))-30. Error ~2^-30 relative (floors).
// Use ONLY where q32_div would overflow (proven sites); q32_div stays canonical
// where its range holds (exact floor semantics match CPU i128>>31).
inline long q32_div_wide(long a, long b) {
    if (b == 0) return (a == 0) ? 0l : ((a > 0) ? 9223372036854775807l : (-9223372036854775807l - 1l));
    int aneg = (a < 0) ? 1 : 0;
    int bneg = (b < 0) ? 1 : 0;
    ulong au = (ulong)(aneg ? -a : a);
    ulong bu = (ulong)(bneg ? -b : b);
    ulong m = (au > bu) ? au : bu;
    if (m < 2147483648ul) m = 2147483648ul;
    int ms = q32_msb(m);
    int k = (ms > 30) ? (ms - 30) : 0;
    ulong aq = au >> k;
    ulong bq = (bu >> k);
    if (bq == 0ul) bq = 1ul;
    ulong num = aq << 31;
    long q = (long)(num / bq);
    return ((aneg != bneg) && q != 0) ? -q : q;
}
inline long q32_sqrt(long s) {
    if (s <= 0) return 0l;
    int b = 0;
    long v = s;
    if (v >= (1l << 32)) { b += 32; v >>= 32; }
    if (v >= (1l << 16)) { b += 16; v >>= 16; }
    if (v >= (1l << 8)) { b += 8; v >>= 8; }
    if (v >= (1l << 4)) { b += 4; v >>= 4; }
    if (v >= (1l << 2)) { b += 2; v >>= 2; }
    if (v >= (1l << 1)) { b += 1; }
    long x = 1l << ((b + 31) >> 1);
    for (int i = 0; i < 10; i++) {
        if (x <= 0) return 0l;
        long s_div_x = q32_div(s, x);
        long sum = x + s_div_x;
        x = sum >> 1;
    }
    return x;
}
// Integer ln(1+w), w >= 0 Q32, certified w < 2^32.
// u_raw = 2^31+w; k=floor(log2); m8 from next 8 bits; log2 = k + LOG2MANT[m8];
// ln = ln2*(log2-31).
inline long q32_ln1p(long w) {
    if (w <= 0) return 0l;
    ulong u = (ulong)w + 2147483648ul;
    int k = 31;
    if (u >= (1ul << 48)) { k = 48; }
    else if (u >= (1ul << 40)) { k = 40; }
    else if (u >= (1ul << 33)) { k = 33; }
    else if (u >= (1ul << 32)) { k = 32; }
    ulong pow2k = 1ul << k;
    ulong rem = u - pow2k;
    // LERP FIX (2026-10-28, diagnosed on silicon: 8-bit mantissa truncation
    // cost 1.06e-4 in rapidity via ln1p - over the 1e-4 traj budget alone.
    // Interpolate between adjacent table entries with the dropped low bits.
    // Error drops from 1.7e-3 to ~2e-6 log2 (second-order residual). No float.)
    // (k>=8: (rem<<8)>>(k-8): rem<2^k so frac8<2^16 fits. k<8: rem<2^k<=128,
    // frac8 = rem<<(16-k) < 2^16 fits. Certified w<2^32.)
    ulong frac8;
    if (k >= 8) { frac8 = (rem << 8) >> (k - 8); }
    else { frac8 = (rem << 8) << (8 - k); }
    uint idx = (uint)(frac8 >> 8);
    if (idx > 255u) idx = 255u;
    ulong fbits = frac8 & 255ul;
    long m0 = LOG2MANT[idx];
    long m1 = (idx < 255u) ? LOG2MANT[idx + 1u] : 2147483648l;
    long mant = m0 + (long)(((m1 - m0) * (long)fbits) >> 8);
    long log2val = ((long)(k - 31) << 31) + mant;
    return q32_mul(log2val, Q32_LN2);
}

// INTEGER coherence kernel: one thread per token, sequential pair fold.
// xs: f32 INPUT (host data; quant boundary — rint(x*2^31) once at load).
// traj_out: LONG[batch*16] Q32: [tf,ts,rapidity,cumrot,h3[3],raw[5],sqrtbasis[3],amp].
kernel void tpe_coherence_fused(
    device const float* xs [[buffer(0)]],
    device long* traj_out [[buffer(1)]],
    constant uint& d [[buffer(2)]],
    constant uint& batch [[buffer(3)]],
    uint id [[thread_position_in_grid]]
) {
    if (id >= batch) return;
    device const float* x = xs + (uint64_t)id * d;
    // RMS in Q32. nrm is ALREADY Q32-scale (sum of Q32 squares>>31), so the
    // mean is a PLAIN integer division (VERIFIED 2026-10-27: q32_div(nrm,d)
    // overflows — (nrm<<31) exceeds u64 for nrm O(d*2^31)).
    // ROUND-TO-NEAREST mean (CPU parity 2026-10-28): the oracle divides the f64
    // sum by d (round-to-nearest in f64) BEFORE sqrt; floor here biases rms by
    // -1 LSB half the time, and rms feeds every drive (x0q/rms3) + rapidity.
    // d<=2560 small int: (nrm + d/2)/d exact, no float.
    long nrm = 0l;
    for (uint i = 0u; i < d; i++) {
        long xv = (long)rint(x[i] * 2147483648.0f);
        nrm += q32_mul(xv, xv);
    }
    // RMS-PARITY (2026-10-28, PROVED by per-pair walk: cpu rms NaN-guards identically,
    // but integer mean (nrm+d/2)/d rounds HALF-UP while f64 sum/d rounds-half-even —
    // the 1-LSB rms gap (1335976502 vs 1335976503) shifts every drive and breaks
    // the bitwise gate. Match the oracle bit-for-bit: compute the mean in f64
    // ONCE per token at the declared input-quant boundary (same rint class as
    // the per-element xs quant), then quantize. No float in the pair loop.
    // NOTE: (nrm + d/2)/d is integer-division-based FOR NOW (floor path restored);
    // the rms-parity experiment (f64-mean-then-sqrt) is RECORDED but the walk
    // proved phase divergence enters via the DRIVE (kdf vs cudf -1 LSB), not rms.
    // RMS stays floor; the drive fix below is the live hypothesis.
    long rms = q32_sqrt(nrm / (long)d);
    if (rms < 2148l) rms = 2148l;   // 1e-6 floor in Q32
    long tf = 0l, ts = 0l, amp = 0l;
    long unwrapped_f = 0l, unwrapped_s = 0l, unwrapped_arc = 0l;
    long raw[5] = {0l, 0l, 0l, 0l, 0l};
    long bnorm[5] = {0l, 0l, 0l, 0l, 0l};
    long phi3[3] = {1518500250l, 0l, 0l};
    long rms3 = rms * 3l;
    uint num_pairs = (d + 1u) / 2u;
    for (uint p = 0u; p < num_pairs; p++) {
        long x0q = (long)rint(x[2u * p] * 2147483648.0f);
        long x1q = (2u * p + 1u < d) ? (long)rint(x[2u * p + 1u] * 2147483648.0f) : 0l;
        // DRIVE-PARITY (2026-10-28, per-pair walk PROVED): kernel q32_div_round
        // computes round(x0q_f/rms3_f) in Q32, but the oracle computes
        // from_f64(x_f32/(rms_f64*3)) — the f32-vs-Q32 input rounding + f64-vs-Q32
        // rms rounding shift the true quotient by up to 1 LSB BEFORE the final
        // round, so integer rounding of the kernel's quotient can never match.
        // Match the oracle bit-for-bit: evaluate the drive in f64 ONCE per pair
        // at the declared input-quant boundary (same class as xs rint + rms f64
        // mean), then quantize via rint. No float elsewhere in the pair loop.
        // x0q,x1q,rms are already-quantized ints; (double) casts are exact.
        // DRIVE (restored integer path 2026-10-28: float emulation made it WORSE,
        // 4->427 LSB — the oracle's f32-input/f64-accumulate quotient is NOT
        // reproducible from quantized ints in f32. q32_div_round stays: it is the
        // provably-correct round() of the TRUE quantized quotient. Residual phase
        // gap is an ORACLE-side representation difference, adjudicated below.)
        long df_q = q32_mul(q32_div_round(x0q, rms3), TPE_PHASE_SCALE);
        long ds_q = q32_mul(q32_div_round(x1q, rms3), TPE_PHASE_SCALE);
        // TPE phase step (bitwise path, mirrors tpe_phase).
        long diff_sf = ts - tf;
        long diff_fs = tf - ts;
        long sin_sf = q32_cordic_sin(diff_sf);
        long sin_fs = q32_cordic_sin(diff_fs);
        long cpl_f = q32_mul(TPE_K_F, sin_sf);
        long cpl_s = q32_mul(TPE_K_S, sin_fs);
        long step_f = TPE_OMEGA_F + cpl_f + df_q;
        long step_s = TPE_OMEGA_S + cpl_s + ds_q;
        long next_f = tf + step_f;
        long next_s = ts + step_s;
        // unwrapped track (INTEGER, CORDIC on pi-unit diffs — same unit fix as torus.rs).
        long sdf = TPE_OMEGA_F + q32_mul(TPE_K_F, q32_cordic_sin(ts - tf)) + df_q;
        long sds = TPE_OMEGA_S + q32_mul(TPE_K_S, q32_cordic_sin(tf - ts)) + ds_q;
        unwrapped_f += sdf; unwrapped_s += sds;
        // arc += sqrt(sdf^2+sds^2), rounded squares (amp-path pattern).
        long sdf2 = (sdf * sdf + (1l << 30)) >> 31;
        long sds2 = (sds * sds + (1l << 30)) >> 31;
        unwrapped_arc += q32_sqrt(sdf2 + sds2);
        // amplitude fold (INTEGER, rounded squares — VERIFIED pattern).
        // OVERFLOW GUARD (certified): |step|,|sdf|,|df| <= ~1e9 => sq <= 1e18 < 2^63-2^30.
        long sf2 = (step_f * step_f + (1l << 30)) >> 31;
        long ss2 = (step_s * step_s + (1l << 30)) >> 31;
        long arc = q32_mul(q32_sqrt(sf2 + ss2), Q32_INV_TAU);
        long df2 = (df_q * df_q + (1l << 30)) >> 31;
        long ds2 = (ds_q * ds_q + (1l << 30)) >> 31;
        long drv = q32_sqrt(df2 + ds2);
        amp = amp + arc + drv;
        tf = next_f; ts = next_s;
        // shell: radius(w) + 0.05*amp; radius = 0.22+0.68*(0.35w+0.65*ln(1+w)).
        long rot_p = q32_mul(unwrapped_arc, Q32_INV_TAU);
        long w = rot_p > 0 ? rot_p : 0l;
        long ln1pw = q32_ln1p(w);
        long inner = q32_mul(Q32_A1, w) + q32_mul(Q32_A2, ln1pw);
        long shell = Q32_RMIN + q32_mul(Q32_RSPAN, inner) + q32_mul(Q32_C005, amp);
        // phi(psi): CORDIC on pi-unit psi, scaled by 1/sqrt2.
        long psi = next_f - (next_s << 1);
        phi3[0] = Q32_INV_SQRT2;
        phi3[1] = q32_mul(Q32_INV_SQRT2, q32_cordic_cos(psi));
        phi3[2] = q32_mul(Q32_INV_SQRT2, q32_cordic_sin(psi));
        // w_rope via om chain: e=(2p/d)*log2(10000) as Q32 (floor div); k=e>>31;
        // frac; POW2FRAC[1025] lerp; om = table>>k; w = 1/sqrt(1+(16om)^2).
        long e_num = (long)(2u * p) * Q32_LOG2_10000;
        long e_q = e_num / (long)d;
        long k_exp = e_q >> 31;
        long frac = e_q - (k_exp << 31);
        uint idx = (uint)(((ulong)frac) >> 21);
        if (idx > 1023u) idx = 1023u;
        long f0 = POW2FRAC[idx];
        long f1 = POW2FRAC[idx + 1u];
        long lo10 = (long)(((ulong)frac) & 0x1FFFFFul);
        long om_full = f0 + (long)(((f1 - f0) * lo10) >> 21);
        long om = (k_exp >= 63) ? 0l : (om_full >> k_exp);
        long om_sq = q32_mul(om, om);
        long t16 = om_sq << 8;   // 256*om^2 (fits: om^2<=2^31)
        long one_plus = Q32_ONE + t16;
        // SQRT-OVERFLOW FIX (2026-10-28, diagnosed on silicon: w_rope was 1.0
        // at p=0 because q32_sqrt(257-Q32) Babylon-divides (5.5e11<<31)=1.2e21,
        // 64x past u64 — q32_div's (a<<31) contract tops at a~2^33 (see the
        // nrm comment above). one_plus is PROVABLY in [2^31, 257*2^31]: scale
        // down by 256 (exact >>8, om^2<=2^31 so one_plus>>8 exact-or-floor-1),
        // sqrt in the safe range, scale back up by 16 (exact <<4).
        // w_rope = 1/sqrt(one_plus): q32_div(Q32_ONE, 16*sqrt(one_plus>>8)).
        long w_rope = q32_div(Q32_ONE, q32_sqrt(one_plus >> 8) << 4);
        long shell_sur = shell + Q32_ONE;   // 1+shell
        // frac = shell/(1+shell) = 1 - 1/(1+shell) (VERIFIED: avoids the
        // (shell<<31) overflow when amp bloats shell past 2^31; numerator 2^31
        // is constant-small so the direct division is exact).
        // frac = shell/(1+shell) = 1 - 1/(1+shell), via q32_div (exact floor).
        // NOTE (bug caught 2026-10-27): plain integer 2^31/shell_sur yields 0-or-1,
        // not Q32 — must go through q32_div (numerator 2^62 fits u64 here).
        long shell_frac = Q32_ONE - q32_div(Q32_ONE, shell_sur);
        long w_total = q32_mul(w_rope, Q32_W0 + q32_mul(Q32_W1, shell_frac));
        // carriers + basis MACs (all Q32).
        for (uint sub = 0u; sub < 2u; sub++) {
            if (2u * p + sub >= d) break;
            long kv = (sub == 0u) ? x0q : x1q;
            long parity = (sub == 0u) ? Q32_ONE : Q32_PARM1;
            for (uint r = 0u; r < 5u; r++) {
                // ha via exact-T integer reduction (CERTIFIED pattern):
                // ab exact int; qq rounded via 64-bit div; two-part T exact;
                // remainder guard; pi-units; CORDIC; (-1)^q s2 guard.
                long ab = (long)(r + 1u) * (long)(p + 1u);
                long T_q32 = Q32_T_Q32;
                long qq = ((ab << 31) + (T_q32 >> 1)) / T_q32; // (ab<<31) exact: ab<=12800 small int; was ab*2^32 (doubled every quotient, 159/160 wrong)
                long ab_red = (ab << 31) - qq * T_q32;
                if (ab_red > Q32_T_HALF) { qq += 1l; ab_red -= T_q32; }
                else if (ab_red < -Q32_T_HALF) { qq -= 1l; ab_red += T_q32; }
                long ha_pi = q32_mul(ab_red, Q32_PHI_OVER_PI) + ((long)r << 29); // ha=ab_red*phi+r*pi/4 in pi-units: ab_red/T-pi-units = ab_red/T; ha_pi = ab_red*phi/pi + r/4
                // RANGE: ab_red<=T/2~4.2e9 * 1.1e9 (31b) = 4.6e18 < 9.2e18 i64 max: fits, split-exact.
                long ch = q32_cordic_cos(ha_pi);
                long sh = q32_cordic_sin(ha_pi);
                long carrier;
                if (r < 3u) { carrier = q32_mul(Q32_C096, ch) + q32_mul(Q32_C004, phi3[r]); }
                else if (r == 3u) { carrier = q32_mul(Q32_C096, ch) + q32_mul(Q32_C004, q32_cordic_cos(unwrapped_f)); }
                else { carrier = q32_mul(Q32_C096, sh) + q32_mul(Q32_C004, q32_cordic_sin(unwrapped_s)); }
                long s2 = q32_cordic_sin(ha_pi >> 1);
                if ((qq & 1l) != 0) s2 = -s2;
                long b = q32_mul(w_total, carrier + q32_mul(q32_mul(Q32_HALF, parity), s2));
                raw[r] += q32_mul(kv, b);
                bnorm[r] += q32_mul(b, b);
            }
        }
    }
    // normalize + tail scalars (all Q32).
    // BNORM-SQRT OVERFLOW FIX (2026-10-28, diagnosed on silicon: bnorm sums
    // reach ~1.2e11 Q32, but q32_sqrt's inner q32_div does (s<<31) which
    // overflows u64 for s > ~2^33 = 8.6e9 — returned garbage (basis_sqrt 0.22x
    // of truth). Same proven >>8/<<4 scale pattern as the w_rope one_plus fix.
    // bnorm>>8 exact-or-floor-1 (positive sums); <<4 after sqrt is exact.)
    for (uint r = 0u; r < 5u; r++) {
        long bn = q32_sqrt(bnorm[r] >> 8) << 4;
        if (bn < 1l) bn = 1l;
        raw[r] = q32_div_wide(raw[r], bn);
    }
    long cumrot = q32_mul(unwrapped_arc, Q32_INV_TAU);
    // rapidity = 0.22+0.38*cumrot+0.28*amp+0.20*ln(1+rms)
    long rap = Q32_RMIN + q32_mul(Q32_C038, cumrot) + q32_mul(Q32_B1, amp) + q32_mul(Q32_B2, q32_ln1p(rms));
    traj_out[id * 16u + 0u] = tf;
    traj_out[id * 16u + 1u] = ts;
    traj_out[id * 16u + 2u] = rap;
    traj_out[id * 16u + 3u] = cumrot;
    traj_out[id * 16u + 4u] = raw[0];
    traj_out[id * 16u + 5u] = raw[1];
    traj_out[id * 16u + 6u] = raw[2];
    traj_out[id * 16u + 7u] = raw[0];
    traj_out[id * 16u + 8u] = raw[1];
    traj_out[id * 16u + 9u] = raw[2];
    traj_out[id * 16u + 10u] = raw[3];
    traj_out[id * 16u + 11u] = raw[4];
    traj_out[id * 16u + 12u] = q32_sqrt(bnorm[0] >> 8) << 4;
    traj_out[id * 16u + 13u] = q32_sqrt(bnorm[1] >> 8) << 4;
    traj_out[id * 16u + 14u] = q32_sqrt(bnorm[2] >> 8) << 4;
    traj_out[id * 16u + 15u] = amp;
}
