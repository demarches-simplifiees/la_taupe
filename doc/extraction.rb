require 'json'

raw = `pdftotext -layout ants_2d-doc_specifications_techniques_des_codes_a_barres_2d-doc_v330-1.pdf -`

regex = %r{
  ^(?<id>[[:alnum:]]{2})[[:space:]]{3}(?<nom>.*)\n
  ^[[:space:]]{5}Taille\sMin..(?<min>.*)\n
  ^[[:space:]]{5}Taille\sMax..(?<max>.*)\n
  ^[[:space:]]{5}Type[[:space:]]+(?<type>.*)\n
  ^[[:space:]]{5}Description[[:space:]]+(?<description>.*\n^(?:[[:space:]]{11}.*\n)*)
}x

scanned = raw.scan(regex)

result = scanned.map do |match|
  {
    id: match[0].strip,
    nom: match[1].strip,
    min: match[2].strip,
    max: match[3].strip,
    nature: match[4].strip,
    description: match[5].strip.gsub(/\n/, ' ').gsub(/\s+/, ' ')
  }
end

File.open('../src/twoddoc/structure.json', 'w') do |f|
  f.write(JSON.pretty_generate(result))
end
